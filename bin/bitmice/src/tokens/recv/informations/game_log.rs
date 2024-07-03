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
    let c = data.read_u8();
    let cc = data.read_u8();
    let old_c = data.read_u8();
    let old_cc = data.read_u8();
    let error = data.read_utf();

    let client = client.lock().await;

    if (c, cc) == (0, 0) {
        return Ok(());
    }

    if (c, cc) == (1, 1) {
        log::error!(
            "[old] game error encountered by [{}], c = [{}], cc = [{}], error = [{}]",
            client.full_name(),
            old_c,
            old_cc,
            error
        );
    } else {
        log::error!(
            "game error encountered by [{}], c = [{}], cc = [{}], error = [{}]",
            client.full_name(),
            c,
            cc,
            error
        );
    }

    Ok(())
}
