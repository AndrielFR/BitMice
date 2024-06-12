// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let round_code = data.read_i32();
    let _cheese_x = data.read_i16();
    let _cheese_y = data.read_i16();
    let _distance = data.read_i16();

    let mut client = client.lock().await;
    let room = client.room.as_ref().unwrap().lock().await;

    let last_round_code = room.last_round_code as i32;
    drop(room);

    if round_code == last_round_code {
        client.get_cheese().await?;
    }

    Ok(())
}
