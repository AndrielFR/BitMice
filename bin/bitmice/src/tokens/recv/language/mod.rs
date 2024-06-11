// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod language_list;
mod set_language;

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
        1 => set_language::handle(client, server, data, packet_id).await,
        2 => language_list::handle(client, server, data, packet_id).await,
        _ => {
            log::debug!("cc = [{}] not identified\ndata = [{:?}]", cc, data);
            Ok(())
        }
    }
}
