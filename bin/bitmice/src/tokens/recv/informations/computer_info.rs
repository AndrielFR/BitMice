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
    let lang = data.read_utf();
    let os = data.read_utf();

    let mut client = client.lock().await;

    client.computer_lang = lang;
    client.computer_os = os;

    Ok(())
}
