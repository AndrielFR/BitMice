// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{
    io::Read,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
    vec,
};

use bitmice_utils::{ByteArray, OldData};
use rand::{seq::SliceRandom, Rng};
use tokio::sync::Mutex;

use crate::{tokens, Client, Result};

const VANILLA_MAPS_FOLDER: &str = "./assets/maps/vanilla/";

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub map_name: String,
    pub lang: String,
    pub map_xml: String,
    pub next_map: String,
    sync_name: String,

    pub map_code: i32,
    pub map_perma: i8,
    pub round_time: i16,
    pub start_time: u128,
    pub time_change_map: u16,
    pub last_round_code: i8,
    sync_code: i32,

    pub can_change_map: bool,
    pub is_new: bool,
    pub is_inverted_map: bool,
    pub is_specific_map: bool,

    clients: Vec<Arc<Mutex<Client>>>,
    pub map_type: MapType,
    pub room_type: RoomType,
}

impl Room {
    pub fn new(name: String, lang: String) -> Self {
        Self {
            name,
            map_name: String::from("BitMice"),
            lang,
            map_xml: String::new(),
            next_map: String::from("-1"),
            sync_name: String::new(),

            map_code: -1,
            map_perma: 0,
            round_time: -1,
            start_time: 0,
            time_change_map: 0,
            last_round_code: -1,
            sync_code: -1,

            can_change_map: true,
            is_new: true,
            is_inverted_map: false,
            is_specific_map: false,

            clients: Vec::new(),
            map_type: MapType::Vanilla,
            room_type: RoomType::Vanilla,
        }
    }

    pub fn players(&self) -> Vec<Arc<Mutex<Client>>> {
        self.clients.iter().map(|c| Arc::clone(c)).collect()
    }

    pub async fn alive(&self) -> i16 {
        let mut count = 0;

        for player in self.players() {
            let p = player.lock().await;

            if !p.is_dead {
                count += 1;
            }
        }

        count
    }

    fn select_map(&mut self) {
        if &self.next_map == "-1" {
            match self.map_type {
                MapType::Vanilla => {
                    let (map_code, xml) = self.get_vanilla_map_xml();

                    self.map_code = map_code;
                    self.map_name = String::from("BitMice");
                    self.map_xml = xml;
                    self.map_perma = 22;
                    self.is_inverted_map = false;
                }
                _ => {
                    self.map_code = -1;
                    self.map_name = String::from("Invalid");
                    self.map_xml = String::from("<C><P /><Z><S /><D /><O /></Z></C>");
                    self.map_perma = -1;
                    self.is_inverted_map = false;
                }
            }

            return;
        }

        let next_map = self.next_map.clone();
        self.next_map = String::from("-1");
        self.map_code = -1;

        if let Ok(next_code) = next_map.parse::<i32>() {
            self.map_code = next_code;
        } else if next_map.starts_with("@") {
            // custom
            let map_code = next_map[1..].parse::<i32>().unwrap();

            if let Some(info) = get_map_info(map_code) {
                self.map_code = map_code;
                self.map_name = info[0].to_string();
                self.map_xml = info[1].to_string();
                self.map_perma = info[2].parse::<i8>().unwrap();
                self.map_type = MapType::Custom;
                self.is_inverted_map = false;
            } else {
                self.map_code = 0;
            }
        } else if next_map.starts_with("#") {
            // perm
            let map_perma = next_map[1..].parse::<i8>().unwrap();

            self.map_code = -1;
            self.map_perma = map_perma;
            self.map_type = MapType::Perm;
        } else if next_map.starts_with("<") {
            // xml
            let xml = next_map;

            self.map_code = 0;
            self.map_name = String::from("#Module");
            self.map_xml = xml;
            self.map_perma = 22;
            self.map_type = MapType::Xml;
            self.is_inverted_map = false;
        } else {
            return;
        }
    }

    pub async fn start_map(&self, start: bool) -> Result {
        self.send_data(
            tokens::send::MAP_START_TIMER,
            ByteArray::new().write_bool(start), // use start variable
        )
        .await?;

        Ok(())
    }

    fn get_vanilla_map_xml(&self) -> (i32, String) {
        let mut xml = String::new();

        let file_list = std::fs::read_dir(VANILLA_MAPS_FOLDER)
            .unwrap()
            .map(|m| m.unwrap())
            .collect::<Vec<std::fs::DirEntry>>();
        let path_list = file_list
            .iter()
            .map(|f| f.path())
            .collect::<Vec<std::path::PathBuf>>();
        let code_list = path_list
            .iter()
            .map(|p| {
                p.to_str()
                    .unwrap()
                    .split_terminator("/")
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .split(".")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap()
            })
            .collect::<Vec<i32>>();
        let map_code = code_list.choose(&mut rand::thread_rng()).unwrap();
        let file = std::fs::File::open(format!("{}{}.xml", VANILLA_MAPS_FOLDER, map_code));
        if let Ok(mut f) = file {
            f.read_to_string(&mut xml).unwrap();
        }

        (*map_code, xml)
    }

    pub async fn get_sync_code(&mut self) -> i32 {
        let players = self.players();

        if players.is_empty() {
            if self.sync_code == -1 {
                self.sync_code = 0;
                self.sync_name = String::new();
            }
        } else {
            let player = players.get(rand::thread_rng().gen_range(0..players.len()));
            if let Some(player) = player {
                let p = player.lock().await;

                self.sync_code = p.id as i32;
                self.sync_name = p.full_name();
            }
        }

        self.sync_code
    }

    pub async fn add_client(&mut self, client: Arc<Mutex<Client>>) -> Result {
        if !self.is_new {
            let mut c = client.lock().await;

            c.is_dead = true;
            let client_id = c.id;
            let b = ByteArray::new()
                .write_bytes(c.player_data())
                .write_bool(false)
                .write_bool(true);
            drop(c);

            self.send_data_except(client_id, tokens::send::PLAYER_RESPAWN, b)
                .await?;
            crate::client::start_play(Arc::clone(&client)).await?;
        }

        self.clients.push(client);

        Ok(())
    }

    pub async fn remove_client(&mut self, client_id: u32) {
        for (i, client) in self.players().iter().enumerate() {
            let mut c = client.lock().await;
            if c.id == client_id {
                c.reset_player();
                c.is_dead = true;
                c.score = 0;

                if self.alive().await <= 0 {
                    // TODO: delete room
                }

                self.send_old_data(
                    tokens::old::send::PLAYER_DISCONNECT,
                    vec![OldData::Integer(c.id as i32)],
                )
                .await
                .unwrap();

                self.clients.swap_remove(i);
                break;
            }
        }
    }

    pub async fn send_data(&self, tokens: (u8, u8), data: ByteArray) -> Result {
        for player in self.players() {
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
        for player in self.players() {
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

    pub async fn send_old_data(&self, tokens: (u8, u8), data: Vec<OldData>) -> Result {
        for player in self.players() {
            match player.try_lock() {
                Ok(mut player) => player.send_old_data(tokens, data.clone()).await?,
                Err(_) => continue,
            }
        }

        Ok(())
    }
}

pub async fn change_map(room: Arc<Mutex<Room>>) -> Result {
    let mut r = room.lock().await;

    r.sync_name = String::new();

    r.round_time = 120;
    r.last_round_code = (r.last_round_code + 1) % 127;
    r.sync_code = -1;

    r.can_change_map = true;

    if r.name.starts_with("\x03[Editeur] ") {
        r.map_type = MapType::Editor;
        r.round_time = 0;
    } else if r.name.starts_with("\x03[Tutorial] ") {
        r.map_code = 900;
        r.is_specific_map = true;
        r.map_type = MapType::Tutorial;
        r.round_time = 0;
    } else if r.name.starts_with("\x03[Totem] ") {
        r.map_code = 444;
        r.is_specific_map = true;
        r.map_type = MapType::Totem;
        r.round_time = 0;
    } else {
        r.select_map();

        if r.name.starts_with("bootcamp") {
            r.room_type = RoomType::Bootcamp;
            r.round_time = 360;
        } else if r.name.starts_with("defilante") {
            r.room_type = RoomType::Defilante;
        } else if r.name.starts_with("racing") {
            r.room_type = RoomType::Racing;
            r.round_time = 63;
        } else if r.name.starts_with("survivor") {
            r.room_type = RoomType::Survivor;
        }
    }
    r.start_time = UNIX_EPOCH.elapsed().unwrap().as_millis();

    let players = r.players();
    drop(r);
    for player in players {
        let mut p = player.lock().await;

        p.reset_player();
        drop(p);
        crate::client::start_play(player).await?;
    }

    let mut r = room.lock().await;
    r.can_change_map = false;

    Ok(())
}

pub async fn trigger(room: Arc<Mutex<Room>>) -> Result {
    tokio::spawn(async move {
        loop {
            let mut r = room.lock().await;

            r.round_time -= 1;

            let alive_count = r.alive().await;
            let round_time = r.round_time;

            let can_change_map = r.can_change_map;
            let is_new = r.is_new;

            let map_type = r.map_type;
            let room_type = r.room_type;
            drop(r);

            if room_type == RoomType::Racing && can_change_map && round_time > 20 {
                let mut r = room.lock().await;
                r.round_time = 21;

                for player in r.players() {
                    let mut p = player.lock().await;

                    p.send_data(
                        tokens::send::ROUND_TIME,
                        ByteArray::new().write_i16(r.round_time),
                    )
                    .await
                    .unwrap();
                }
            }

            if is_new || alive_count <= 0 || can_change_map && round_time <= 0 {
                if is_new {
                    let mut r = room.lock().await;
                    r.is_new = false;
                }

                if map_type == MapType::Editor
                    || map_type == MapType::Totem
                    || map_type == MapType::Tutorial
                {
                    break;
                }
                crate::room::change_map(Arc::clone(&room)).await.unwrap();
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MapType {
    Custom,
    Editor,
    Perm,
    Totem,
    Tutorial,
    Vanilla,
    Xml,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RoomType {
    Bootcamp,
    Defilante,
    Racing,
    Survivor,
    Vanilla,
}

fn get_map_info<'a>(_map_code: i32) -> Option<Vec<&'a str>> {
    Some(vec![
        "Euzinho",
        r#"<C><P F="7" /><Z><S><S Y="275" T="10" P="0,0,0.3,0,0,0,0,0" L="120" X="60" H="300" /><S Y="315" T="11" P="1,0,0.05,0.1,0,0,0,0" c="1" L="20" X="210" H="275" /><S Y="275" T="10" P="0,0,0.3,0,0,0,0,0" L="120" H="300" X="740" /><S Y="475" T="8" P="0,0,0.2,0.2,0,0,0,0" L="400" X="400" H="40" /><S Y="315" T="11" P="1,0,0.05,0.1,0,0,0,0" H="275" L="20" X="340" c="1" /><S Y="315" T="11" P="1,0,0.05,0.1,0,0,0,0" c="1" L="20" X="470" H="275" /><S Y="315" T="11" P="1,0,0.05,0.1,0,0,0,0" H="275" L="20" X="590" c="1" /></S><D><T Y="127" X="56" /><F Y="123" X="740" /></D><O /></Z></C>"#,
        "22",
    ])
}
