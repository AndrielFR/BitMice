// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

use crate::{tokens, Client, Result, Server};

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let crouch = data.read_i8();

    let client = client.lock().await;
    let room = Arc::clone(&client.room.as_ref().unwrap());
    let r = room.lock().await;

    let b = ByteArray::new()
        .write_i32(client.id)
        .write_i8(crouch)
        .write_i8(0);

    r.send_data(tokens::send::CROUCH, b).await?;

    Ok(())
}
