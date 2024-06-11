// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use bitmice_utils::{bytes_to_string, str_to_bytes, ByteArray};
use once_cell::sync::Lazy;
use std::{io, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{mpsc, Mutex},
};

use crate::{tokens, Client, Result, Room};

pub static CLIENTS: Lazy<Mutex<Vec<Arc<Mutex<Client>>>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static ROOMS: Lazy<Mutex<Vec<Arc<Mutex<Room>>>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Default, Debug)]
pub struct Server {
    pub last_player_id: i32,
}

impl Server {
    pub fn new_player_id(&mut self) -> i32 {
        self.last_player_id += 1;
        self.last_player_id
    }

    pub async fn players(&self) -> Vec<Arc<Mutex<Client>>> {
        let clients = CLIENTS.lock().await;

        let mut ps = Vec::new();
        for client in clients.iter() {
            ps.push(Arc::clone(client));
        }

        ps
    }

    pub async fn get_player(&self, name: String) -> Option<Arc<Mutex<Client>>> {
        let clients = CLIENTS.lock().await;

        for client in clients.iter() {
            let c = client.lock().await;

            if *c.name == name {
                return Some(Arc::clone(client));
            }
        }

        None
    }

    pub async fn rooms(&self) -> Vec<Arc<Mutex<Room>>> {
        let rooms = ROOMS.lock().await;

        let mut rs = Vec::new();
        for room in rooms.iter() {
            rs.push(Arc::clone(room));
        }

        rs
    }

    pub async fn add_room(&self, room: Arc<Mutex<Room>>) {
        let mut rooms = ROOMS.lock().await;
        rooms.push(room);
    }

    pub async fn remove_room(&self, room: Arc<Mutex<Room>>) {
        let mut rooms = ROOMS.lock().await;

        if let Some(index) = rooms
            .iter()
            .position(|v| std::ptr::eq(v.as_ref(), room.as_ref()))
        {
            rooms.swap_remove(index);
        }
    }

    pub async fn get_room(&self, name: String, lang: String) -> Option<Arc<Mutex<Room>>> {
        let rooms = ROOMS.lock().await;

        for room in rooms.iter() {
            let r = room.lock().await;

            if *r.name == name && *r.lang == lang {
                return Some(Arc::clone(room));
            }
        }

        None
    }

    pub async fn get_recommended_room(&self, lang: String) -> Arc<Mutex<Room>> {
        let mut rooms = ROOMS.lock().await;

        let mut last_room: Option<Arc<Mutex<Room>>> = None;
        for room in rooms.iter() {
            let r = room.lock().await;

            if r.lang == lang {
                if let Some(ref l_r) = last_room {
                    let l_r = l_r.lock().await;

                    if r.players_count().await > l_r.players_count().await {
                        drop(l_r);
                        last_room = Some(Arc::clone(room));
                    }
                } else {
                    last_room = Some(Arc::clone(room));
                }
            }
        }

        if last_room.is_none() {
            let room = Arc::new(Mutex::new(Room::new("1".to_string(), lang, true)));
            let r = Arc::clone(&room);
            rooms.push(room);
            return r;
        }

        last_room.unwrap()
    }

    pub async fn send_data(&self, tokens: (u8, u8), data: ByteArray) -> Result {
        for player in self.players().await {
            match player.try_lock() {
                Ok(mut player) => player.send_data(tokens, data.clone()).await?,
                Err(_) => continue,
            }
        }

        Ok(())
    }

    pub async fn send_data_except(
        &self,
        client_id: i32,
        tokens: (u8, u8),
        data: ByteArray,
    ) -> Result {
        for player in self.players().await {
            match player.try_lock() {
                Ok(mut player) => {
                    if player.id != client_id {
                        player.send_data(tokens, data.clone()).await?
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(())
    }
}

pub async fn handle_client(
    client: Client,
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
) {
    let mut clients = CLIENTS.lock().await;
    let player = Arc::new(Mutex::new(client));
    clients.push(Arc::clone(&player));

    tokio::spawn(async move {
        let client: Arc<Mutex<Client>> = Arc::clone(&player);
        let mut client = client.lock().await;

        let (data_tx, mut data_rx) = mpsc::channel(10);
        client.data_sender = Some(data_tx.clone());
        drop(client);

        // reader
        tokio::spawn(async move {
            loop {
                let mut buf = vec![0; 2048];

                match reader.lock().await.read(&mut buf[..]).await {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }

                        let mut data: ByteArray = buf.into();
                        if data.is_empty() {
                            continue;
                        }

                        // parser
                        let player = Arc::clone(&player);
                        let data_tx = data_tx.clone();
                        tokio::spawn(async move {
                            let mut client = player.lock().await;

                            if client.is_closed {
                                return;
                            }

                            log::trace!("received data = [{}]", bytes_to_string(data.as_bytes()));
                            if bytes_to_string(data.as_bytes()).contains("<policy-file-request/>") {
                                let policy = str_to_bytes("<cross-domain-policy><allow-access-from domain=\"*\" to-ports=\"*\"/></cross-domain-policy>");
                                let _ = data_tx.send(ByteArray::with(policy)).await.unwrap();
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                client.close().await.unwrap();
                            }

                            let mut id = 0;
                            let mut length = 0;

                            let mut byte = data.read_u8() & 0xFF;
                            length = length | ((byte & 0x7F) << (id * 7));
                            id += 1;

                            while (byte & 128) == 128 && id < 5 {
                                if data.is_empty() {
                                    continue;
                                }

                                byte = data.read_u8() & 0xFF;
                                length = length | ((byte & 0x7F) << (id * 7));
                                id += 1;
                            }

                            length += 1;

                            if length == 0 {
                                return;
                            } else if length > (data.len() as u8) {
                                length = data.len() as u8;
                            }

                            data = data.read(length.into());
                            client.received_data = data.clone();

                            let packet_id = data.read_u8();
                            client.packet_id = (client.packet_id + 1) % 100;

                            let c = data.read_u8();
                            let cc = data.read_u8();
                            if c > 0 && cc > 0 {
                                let tokens = (c, cc);

                                if tokens != (4, 4) {
                                    log::trace!(
                                        "received packet id = [{}], tokens = {:?}",
                                        client.packet_id,
                                        tokens
                                    );
                                }

                                if tokens != (26, 26) {
                                    client.update_last_response();
                                }

                                let server = Arc::clone(&client.server);
                                drop(client);
                                let client = Arc::clone(&player);

                                tokens::recv::parse_tokens(client, server, tokens, data, packet_id)
                                    .await
                                    .unwrap();
                            }
                        });
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(e) => log::error!("error reading data: {:?}", e),
                }
            }
        });

        // writer
        tokio::spawn(async move {
            while let Some(data) = data_rx.recv().await {
                match writer.lock().await.write_all(data.as_bytes()).await {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                    Err(e) => log::error!("error writing data: {:?}", e),
                }
            }
        });
    });
}
