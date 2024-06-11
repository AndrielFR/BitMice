// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{tokens, Client, Result, Server};
use bitmice_utils::{language_code, language_info, ByteArray};
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let langue = data.read_utf().to_lowercase();
    let lang = if langue.len() == 2 {
        language_info(&langue).1
    } else {
        language_code(&langue)
    };

    let mut client = client.lock().await;

    client
        .send_data(
            tokens::send::SET_LANGUAGE,
            ByteArray::new()
                .write_utf(&lang)
                .write_utf(language_info(&lang).1)
                .write_i16(0)
                .write_bool(false)
                .write_bool(true)
                .write_utf(""),
        )
        .await?;

    Ok(())
}
