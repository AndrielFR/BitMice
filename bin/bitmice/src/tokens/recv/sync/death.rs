// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{client, Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let round_code = data.read_i32();

    let c = client.lock().await;
    let room = c.room.as_ref().unwrap();
    let r = room.lock().await;

    let last_round_code = r.last_round_code as i32;
    drop(r);
    drop(c);

    if round_code == last_round_code {
        client::die(Arc::clone(&client)).await?;
    }

    Ok(())
}
