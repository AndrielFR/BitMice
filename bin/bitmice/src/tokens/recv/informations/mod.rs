// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod computer_info;
mod correct_version;
mod game_log;

use std::sync::Arc;

use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

use crate::{Client, Result, Server};

pub async fn parse_token(
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    cc: u8,
    data: ByteArray,
    packet_id: u8,
) -> Result {
    match cc {
        1 => correct_version::handle(client, server, data, packet_id).await,
        4 => game_log::handle(client, server, data, packet_id).await,
        17 => computer_info::handle(client, server, data, packet_id).await,
        _ => {
            log::debug!("cc = [{}] not identified\ndata = [{:?}]", cc, data);
            Ok(())
        }
    }
}
