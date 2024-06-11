// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{tokens, Client, Result, Server};
use bitmice_utils::{language_info, language_list, ByteArray};
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    _data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut client = client.lock().await;

    let langs = language_list();

    let mut b = ByteArray::new()
        .write_i16(langs.len() as i16) // language count
        .write_utf(&client.lang) // current language code
        .write_utf(language_info(&client.lang).0) // current language name
        .write_utf(language_info(&client.lang).1); // current language code

    for lang in langs {
        if lang != client.lang {
            let info = language_info(lang);

            b = b.write_utf(lang).write_utf(info.0).write_utf(info.1);
        }
    }

    client.send_data(tokens::send::LANGUAGE_LIST, b).await?;
    Ok(())
}
