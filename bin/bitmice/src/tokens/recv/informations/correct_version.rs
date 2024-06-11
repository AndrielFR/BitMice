// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{tokens, Client, Result, Server};
use bitmice_utils::ByteArray;
use rand::Rng;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let version = data.read_i16();
    let lang = data.read_utf().to_uppercase();
    let ckey = data.read_utf();
    let stand = data.read_utf();

    let mut client = client.lock().await;

    let ip_address = client.address().to_string();

    log::debug!("new client on [{}] using [{}]", ip_address, stand);

    if version != 616 || ckey != "yAdByj" {
        log::debug!(
            "[{}] disconnected by using incorrect version = [{}] and/or ckey = [{}]",
            ip_address,
            version,
            ckey
        );
        client.close().await?;
        return Ok(());
    }

    let auth_key = rand::thread_rng().gen_range(0..2147483647);
    client.auth_key = auth_key;

    client.version_validated = true;
    client
        .send_data(
            tokens::send::CORRECT_VERSION,
            ByteArray::new()
                .write_i32(0) // players count
                .write_utf(&lang) // lang
                .write_utf(&lang) // lang
                .write_i32(auth_key) // auth key
                .write_bool(false),
        )
        .await?;
    client
        .send_data(
            tokens::send::BANNER_LOGIN,
            ByteArray::new()
                .write_i8(1)
                .write_i8(2) // banner id
                .write_i8(1)
                .write_bool(false),
        )
        .await?;
    client
        .send_data(
            tokens::send::IMAGE_LOGIN,
            ByteArray::new().write_utf("x_noel2014.jpg"),
        )
        .await?;
    /* client
    .send_data(
        tokens::send::VERIFY_CODE,
        ByteArray::new().write_i32(rand::thread_rng().gen_range(0..10000)),
    )
    .await?; */

    Ok(())
}
