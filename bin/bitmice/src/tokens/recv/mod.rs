// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod informations;
mod language;
mod login;
mod player;
mod room;
mod sync;

use std::sync::Arc;

use crate::{Client, Result, Server};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn parse_tokens(
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    tokens: (u8, u8),
    data: ByteArray,
    packet_id: u8,
) -> Result {
    let (c, cc) = tokens;

    match c {
        4 => sync::parse_token(client, server, cc, data, packet_id).await,
        5 => room::parse_token(client, server, cc, data, packet_id).await,
        8 => player::parse_token(client, server, cc, data, packet_id).await,
        26 => login::parse_token(client, server, cc, data, packet_id).await,
        28 => informations::parse_token(client, server, cc, data, packet_id).await,
        176 => language::parse_token(client, server, cc, data, packet_id).await,
        _ => {
            log::debug!("tokens {:?} not identified\ndata = [{:?}]", tokens, data);
            Ok(())
        }
    }
}
