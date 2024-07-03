// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use bitmice_utils::{bytes_to_string, str_to_bytes, ByteArray};
use once_cell::sync::Lazy;
use std::{sync::Arc, time::Duration, usize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{mpsc, Mutex},
};

use crate::{room::MapType, tokens, Client, Result, Room};

pub static CLIENTS: Lazy<Mutex<Vec<Arc<Mutex<Client>>>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static ROOMS: Lazy<Mutex<Vec<Arc<Mutex<Room>>>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Debug)]
pub struct Server {
    pub ckey: String,

    pub last_player_id: u32,
    pub version: u16,
}

impl Server {
    pub fn new(ckey: &str, version: u16) -> Self {
        Self {
            ckey: ckey.to_string(),

            last_player_id: 0,
            version,
        }
    }

    pub fn new_player_id(&mut self) -> u32 {
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

    pub async fn get_player(&self, mut name: String) -> Option<Arc<Mutex<Client>>> {
        let clients = CLIENTS.lock().await;

        for client in clients.iter() {
            let c = client.lock().await;

            if c.is_guest {
                name = format!("*{}", name);
            }

            if *c.name == name || *c.full_name() == name {
                return Some(Arc::clone(client));
            }
        }

        None
    }

    pub async fn remove_player(&self, client: Arc<Mutex<Client>>) {
        let mut clients = CLIENTS.lock().await;

        if let Some(index) = clients
            .iter()
            .position(|v| std::ptr::eq(v.as_ref(), client.as_ref()))
        {
            clients.swap_remove(index);
        }
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

            if *r.name == name && *r.lang == lang && r.map_type != MapType::Tutorial {
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

            if !r.name.starts_with("\x03") && r.lang == lang {
                if let Some(ref l_r) = last_room {
                    let l_r = l_r.lock().await;

                    if r.players().len() > l_r.players().len() {
                        drop(l_r);
                        last_room = Some(Arc::clone(room));
                    }
                } else {
                    last_room = Some(Arc::clone(room));
                }
            }
        }

        if last_room.is_none() {
            let room = Arc::new(Mutex::new(Room::new("1".to_string(), lang)));
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
        client_id: u32,
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
        let client = Arc::clone(&player);
        let mut client = client.lock().await;

        let (data_tx, mut data_rx) = mpsc::channel(16);
        client.data_sender = Some(data_tx.clone());
        drop(client);

        // writer
        let writer = writer.clone();
        let player_w = Arc::clone(&player);
        tokio::spawn(async move {
            loop {
                let data = match data_rx.recv().await {
                    Some(d) => d,
                    None => break,
                };

                if let Err(_) = writer.lock().await.write_all(data.as_bytes()).await {
                    log::error!("failed to write data");
                    drop(data_rx);
                    player_disconnect(player_w).await;
                    break;
                }
            }
        });

        // reader
        let reader = reader.clone();
        tokio::spawn(async move {
            loop {
                let mut buffer = vec![0u8; 4096];

                match reader.lock().await.read(&mut buffer).await {
                    Ok(s) => {
                        if s == 0 {
                            drop(data_tx);
                            player_disconnect(Arc::clone(&player)).await;
                            break;
                        }

                        buffer.truncate(s);
                    }
                    Err(_) => {
                        log::error!("failed to read data");
                        drop(data_tx);
                        player_disconnect(Arc::clone(&player)).await;
                        break;
                    }
                };

                let mut data = ByteArray::with(buffer);
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
                    drop(client);

                    let mut length = 0;
                    let mut o = 0;
                    let mut i = 0;
                    loop {
                        if data.is_empty() {
                            return;
                        }

                        let byte = data.read_u8() & 255;
                        length |= (byte & 127) << (i * 7);
                        o = o << 7;
                        i += 1;

                        if !((byte & 128) == 128) && i < 5 {
                            length += 1;
                            break;
                        }
                    }

                    if ((o >> 1) & length) != 0 {
                        length |= o;
                    }

                    let mut length = length as usize;
                    if length == 0 {
                        return;
                    } else if length > data.len() {
                        length = data.len();
                    }

                    data = data.read(length);
                    let mut client = player.lock().await;
                    let packet_id = data.read_u8();
                    client.packet_id = (client.packet_id + 1) % 100;

                    let tokens = (data.read_u8(), data.read_u8());

                    if tokens.0 > 0 && tokens.1 > 0 {
                        if !vec![(26, 26), (4, 4)].contains(&tokens) {
                            log::debug!(
                                "received packet id = [{}], tokens = {:?} from {}",
                                client.packet_id,
                                tokens,
                                client.full_name(),
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
        });
    });
}

async fn player_disconnect(player: Arc<Mutex<Client>>) {
    let client = player.lock().await;
    let client_id = client.id;

    let room = if client.room.is_some() {
        Some(Arc::clone(&client.room.as_ref().unwrap()))
    } else {
        None
    };
    let server = Arc::clone(&client.server);
    drop(client);

    // remove client from room
    if room.is_some() {
        let r = room.unwrap();
        let mut r = r.lock().await;
        r.remove_client(client_id).await;
    }

    // remove client from server
    let s = server.lock().await;
    s.remove_player(Arc::clone(&player)).await;

    // close tcp connection
    let mut client = player.lock().await;
    let _ = client.close().await;
    drop(client);
}
