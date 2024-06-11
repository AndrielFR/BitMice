// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    _client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    _data: ByteArray,
    _packet_id: u8,
) -> Result {
    Ok(())
}
