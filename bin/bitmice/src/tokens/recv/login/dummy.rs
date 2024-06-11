// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{sync::Arc, time::UNIX_EPOCH};

use crate::{Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    _data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut client = client.lock().await;

    let last_response = client.last_response;
    let now = UNIX_EPOCH.elapsed().unwrap().as_millis();

    if now - last_response >= 900000 {
        // 15min
        client.close().await?;
    }

    Ok(())
}
