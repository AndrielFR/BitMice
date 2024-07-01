// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{io, net::SocketAddr, sync::Arc, time::UNIX_EPOCH};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{mpsc::Sender, Mutex},
};

use crate::{
    room::{MapType, RoomType},
    tokens, Result, Room, Server,
};
use bitmice_utils::{encode_xml, ByteArray, OldData};

#[derive(Debug)]
pub struct Client {
    address: SocketAddr,
    pub(super) reader: Arc<Mutex<OwnedReadHalf>>,
    pub(super) writer: Arc<Mutex<OwnedWriteHalf>>,
    pub(super) server: Arc<Mutex<Server>>,
    pub room: Option<Arc<Mutex<Room>>>,
    pub(super) data_sender: Option<Sender<ByteArray>>,

    pub color: String,
    pub name: String,
    pub nick_color: String,
    pub tag: String,
    pub lang: String,
    pub look: String,
    pub computer_lang: String,
    pub computer_os: String,
    pub last_room: String,
    pub shaman_color: String,

    pub id: u32,
    pub auth_key: i32,
    pub gender: u8,
    pub last_response: u128,
    pub(super) packet_id: u8,
    pub position_x: u64,
    pub position_y: u64,
    pub priv_level: i8,
    pub time_played: u64,
    pub title_number: u16,
    pub title_stars: u8,
    pub score: u16,
    pub speed_x: u16,
    pub speed_y: u16,
    pub start_time: u128,

    pub has_cheese: bool,
    pub(super) is_closed: bool,
    pub is_dead: bool,
    pub is_guest: bool,
    pub is_jumping: bool,
    pub is_moving_right: bool,
    pub is_moving_left: bool,
    pub is_shaman: bool,
    pub version_validated: bool,
    pub last_ping: bool,

    pub ping: (u8, u128),
}

impl Client {
    pub fn new(
        address: SocketAddr,
        reader: Arc<Mutex<OwnedReadHalf>>,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        server: Arc<Mutex<Server>>,
    ) -> Self {
        Self {
            address,
            reader,
            writer,
            server,
            room: None,
            data_sender: None,

            color: String::from("95d9d6"),
            name: String::new(),
            nick_color: String::from("953"),
            tag: String::from("0000"),
            lang: String::new(),
            look: String::from("1;0,0,0,0,0,0,0,0,0,0,0"),
            computer_lang: String::new(),
            computer_os: String::new(),
            last_room: String::new(),
            shaman_color: String::from("95fe3f"),

            id: 0,
            auth_key: 0,
            gender: 2,
            last_response: 0,
            packet_id: 0,
            position_x: 0,
            position_y: 0,
            priv_level: 1,
            title_number: 5,
            title_stars: 3,
            time_played: 0,
            score: 0,
            speed_x: 0,
            speed_y: 0,
            start_time: 0,

            has_cheese: false,
            is_closed: false,
            is_dead: true,
            is_guest: false,
            is_jumping: false,
            is_moving_right: false,
            is_moving_left: false,
            is_shaman: false,
            version_validated: false,
            last_ping: false,

            ping: (0, 0),
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn full_name(&self) -> String {
        if self.is_souris() {
            return self.name.clone();
        } else if self.name.is_empty() {
            return self.address.to_string();
        }

        format!("{}#{}", self.name, self.tag)
    }

    pub fn is_souris(&self) -> bool {
        self.is_guest
    }

    pub fn player_data(&self) -> ByteArray {
        ByteArray::new()
            .write_utf(&self.full_name()) // player name
            .write_u32(self.id) // player code
            .write_bool(self.is_shaman) // is shaman
            .write_bool(self.is_dead) // is dead
            .write_u16(self.score) // player score
            .write_bool(self.has_cheese) // has cheese
            .write_u16(self.title_number) // title number
            .write_u8(self.title_stars) // title stars
            .write_u8(self.gender) // gender
            .write_utf("") // ??
            .write_utf(&self.look) // player look
            .write_bool(false) // ??
            .write_u32(u32::from_str_radix(&self.color, 16).unwrap()) // mouse color
            .write_u32(u32::from_str_radix(&self.shaman_color, 16).unwrap()) // shaman color
            .write_u32(0) // ??
            .write_u32(u32::from_str_radix(&self.nick_color, 16).unwrap()) // nick color
    }

    pub async fn enter_room(&mut self, name: &str) -> Result {
        let server_ = Arc::clone(&self.server);
        let server = server_.lock().await;

        // get room
        let room = match server.get_room(name.to_string(), self.lang.clone()).await {
            Some(r) => r,
            None => {
                if name.is_empty() {
                    server.get_recommended_room(self.lang.clone()).await
                } else {
                    let room = Room::new(name.to_string(), self.lang.clone());
                    let r = Arc::new(Mutex::new(room));
                    server.add_room(Arc::clone(&r)).await;
                    r
                }
            }
        };
        drop(server);

        // parse room's name
        let mut name = name.replace("<", "&lt;");
        if !name.starts_with("*") && !(name.len() > 3 && name.contains("-") && self.priv_level >= 7)
        {
            name = format!("{}-{}", self.lang, name);
        }

        let special_rooms = vec!["\x03[Editeur] ", "\x03[Totem] ", "\x03[Tutorial] "];
        for special_room in special_rooms {
            if name.starts_with(special_room) && !name.contains(&self.full_name()) {
                name = format!("{}-{}", self.lang, self.full_name());
            }
        }

        let mut standard_room = false;
        let default_rooms = vec![
            "vanilla",
            "survivor",
            "racing",
            "music",
            "bootcamp",
            "defilante",
            "village",
        ];
        for default_room in default_rooms {
            if name.starts_with(default_room) || name.trim().parse::<i32>().is_ok() {
                standard_room = true;
            }
        }

        // let room be int
        if name.starts_with("*") {
            let mut r = room.lock().await;

            r.name = name[1..].to_string();
            r.lang = "int".to_string();
        }

        // enter room
        self.send_data(
            tokens::send::ENTER_ROOM,
            ByteArray::new()
                .write_bool(standard_room)
                .write_utf(&name)
                .write_utf(if name.starts_with("*") {
                    "int"
                } else {
                    &self.lang
                }),
        )
        .await?;

        // update client game type
        self.send_data(tokens::send::ROOM_SERVER, ByteArray::new().write_i8(0))
            .await?;
        self.send_data(
            tokens::send::ROOM_TYPE,
            ByteArray::new().write_i8(if name.contains("music") { 11 } else { 4 }),
        )
        .await?;

        self.last_room = name.clone();
        self.room = Some(room);

        Ok(())
    }

    pub fn reset_player(&mut self) {
        self.has_cheese = false;
        self.is_dead = false;
        self.is_jumping = false;
        self.is_moving_right = false;
        self.is_moving_left = false;
        self.is_shaman = false;
    }

    pub async fn load_map(&mut self, new_map: bool, custom_map: bool) -> Result {
        let r = self.room.as_mut().unwrap().lock().await;

        let xml = match custom_map {
            true => r.map_xml.clone(),
            false => String::new(),
        };
        let xml = encode_xml(xml).unwrap();

        let data = ByteArray::new()
            .write_i32(if new_map || custom_map {
                r.map_code
            } else {
                -1
            })
            .write_i16(r.players_count().await as i16)
            .write_i8(r.last_round_code)
            .write_i32(xml.len() as i32)
            .write_bytes(xml)
            .write_utf(if new_map {
                ""
            } else if custom_map {
                &r.name
            } else {
                "-"
            })
            .write_i8(if new_map {
                0
            } else if custom_map {
                r.map_perma
            } else {
                100
            })
            .write_bool(if custom_map { r.is_inverted_map } else { false });

        drop(r);
        self.send_data(tokens::send::NEW_MAP, data).await?;

        Ok(())
    }

    async fn sync(&mut self, code: i32) -> Result {
        let room = self.room.as_ref().unwrap().lock().await;

        let old_data = if room.map_code != 1 {
            vec![OldData::Integer(code), OldData::String(String::new())]
        } else {
            vec![OldData::Integer(code)]
        };
        drop(room);

        self.send_old_data(tokens::old::send::SYNC, old_data)
            .await?;

        Ok(())
    }

    pub async fn get_cheese(&mut self) -> Result {
        let mut room = self.room.as_mut().unwrap().lock().await;

        room.can_change_map = true;
        if !self.has_cheese {
            room.send_data(
                tokens::send::PLAYER_GET_CHEESE,
                ByteArray::new().write_u32(self.id).write_bool(true),
            )
            .await?;
            self.has_cheese = true;

            let map_type = room.map_type;
            drop(room);
            if map_type == MapType::Tutorial {
                self.send_data(tokens::send::TUTORIAL, ByteArray::new().write_i8(1))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn send_data(&mut self, tokens: (u8, u8), data: ByteArray) -> Result {
        if self.is_closed {
            return Ok(());
        }

        self.packet_id = (self.packet_id + 1) % 255;
        let mut b = ByteArray::new();
        let mut length = data.len() + 2;
        let mut b2 = ByteArray::new();
        let mut calc = length >> 7;
        while calc != 0 {
            b2 = b2.write_u8(((length & 127) | 128) as u8);
            length = calc.clone();
            calc = calc >> 7;
        }
        b2 = b2.write_u8((length & 127) as u8);

        b = b
            .write_bytes(b2)
            .write_u8(tokens.0)
            .write_u8(tokens.1)
            .write_bytes(data);
        if let Err(_) = self.data_sender.as_ref().unwrap().send(b).await {
            log::error!("failed to send data to writer");
        }

        Ok(())
    }

    pub async fn send_old_data(&mut self, tokens: (u8, u8), old_data: Vec<OldData>) -> Result {
        if self.is_closed {
            return Ok(());
        }

        let mut p = ByteArray::new()
            .write_u8(1)
            .write_u8(tokens.0)
            .write_u8(tokens.1);

        for d in old_data {
            match d {
                OldData::String(s) => p = p.write_bytes(s.as_bytes()),
                OldData::Bool(b) => p = p.write_bool(b),
                OldData::Byte(b) => p = p.write_i8(b),
                OldData::Short(s) => p = p.write_i16(s),
                OldData::Integer(i) => p = p.write_i32(i),
                OldData::Long(l) => p = p.write_i64(l),
            }
        }

        self.send_data((1, 1), ByteArray::new().write_utf(p.as_str()))
            .await
    }

    pub async fn close(&mut self) -> io::Result<()> {
        let writer = self.writer.clone();

        self.is_closed = true;
        writer.lock_owned().await.shutdown().await?;
        Ok(())
    }

    pub fn update_last_response(&mut self) {
        self.last_response = UNIX_EPOCH.elapsed().unwrap().as_millis();
    }
}

pub async fn die(client_: Arc<Mutex<Client>>) -> Result {
    let mut client = client_.lock().await;

    client.score += 1;
    client.has_cheese = false;
    client.is_dead = true;

    let room = client.room.clone();
    let r = room.as_ref().unwrap().lock().await;

    let b = vec![OldData::Integer(client.id as i32), OldData::Short(client.score as i16)];
    drop(client);
    r.send_old_data(tokens::old::send::PLAYER_DIED, b).await?;

    Ok(())
}

pub async fn start_play(client_: Arc<Mutex<Client>>) -> Result {
    let mut client = client_.lock().await;

    // load map
    client.start_time = UNIX_EPOCH.elapsed().unwrap().as_millis();

    let room = client.room.as_ref().unwrap().lock().await;
    let mut new_map = true;
    let mut custom_map = false;
    if room.map_code != -1 {
        custom_map = true;
    } else if room.map_type == MapType::Editor {
        new_map = false;
    }

    drop(room);
    client.load_map(new_map, custom_map).await?;

    // update player list
    let room = client.room.clone();
    drop(client);
    let mut r = room.as_ref().unwrap().lock().await;
    let players = r.players().await;

    let mut data = ByteArray::new().write_i16(players.len() as i16);
    for player in players {
        let player = player.lock().await;

        data = data.write_bytes(player.player_data());
    }

    let mut client = client_.lock().await;
    client.send_data(tokens::send::PLAYER_LIST, data).await?;
    drop(client);

    // sync users
    let sync_code = r.get_sync_code().await;
    drop(r);
    let mut client = client_.lock().await;
    client.sync(sync_code).await?;

    // update round time
    let r = room.as_ref().unwrap().lock().await;
    let round_time = r.round_time;
    drop(r);
    client
        .send_data(
            tokens::send::ROUND_TIME,
            ByteArray::new().write_i16(if round_time < 0 { 0 } else { round_time }),
        )
        .await?;

    // map start time
    let r = room.as_ref().unwrap().lock().await;
    let is_dead = client.is_dead;
    drop(client);
    if is_dead
        || r.map_type == MapType::Tutorial
        || r.map_type == MapType::Totem
        || r.room_type == RoomType::Bootcamp
        || r.room_type == RoomType::Defilante
        || r.players_count().await < 2
    {
        r.start_map(false).await?;
    } else {
        r.start_map(true).await?;
    }

    Ok(())
}
