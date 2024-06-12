// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{client, Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client_: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let round_code = data.read_i32();
    let _loc_1 = data.read_i8();

    let client = client_.lock().await;
    let room = client.room.as_ref().unwrap().lock().await;

    let last_round_code = room.last_round_code as i32;
    drop(room);
    drop(client);

    if round_code == last_round_code {
        client::die(Arc::clone(&client_)).await?;
    }

    Ok(())
}
