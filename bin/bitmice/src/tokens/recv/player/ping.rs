// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{sync::Arc, time::UNIX_EPOCH};

use crate::{tokens, Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    _data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut client = client.lock().await;

    let now = UNIX_EPOCH.elapsed().unwrap().as_millis();

    if now - client.ping.1 >= 5 {
        client.ping.1 = UNIX_EPOCH.elapsed().unwrap().as_millis();
        client.last_ping = !client.last_ping;

        let b = ByteArray::new()
            .write_u8(client.ping.0)
            .write_bool(client.last_ping);
        client.send_data(tokens::send::PING, b).await?;

        client.ping.0 += 1;
        if client.ping.0 == 31 {
            client.ping.0 = 0;
        }
    }

    Ok(())
}
