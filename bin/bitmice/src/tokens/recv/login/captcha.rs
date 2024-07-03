// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{tokens, Client, Result, Server};
use bitmice_utils::{encode_zlib, generate_captcha, generate_captcha_image, ByteArray};
use rand::Rng;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    _data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut client = client.lock().await;

    let code = generate_captcha(rand::thread_rng().gen_range(3..6));
    let (captcha, width, height) = generate_captcha_image(&code);
    let mut p = ByteArray::new()
        .write_u8(0)
        .write_u16(width as u16)
        .write_u16(height as u16)
        .write_u16((width * height) as u16);

    for row in 0..height {
        for col in 0..width {
            let b = captcha.get_pixel(col, row).0[2];
            p = p.write_u32(b as u32);
        }
    }

    let encoded_captcha = encode_zlib(p.to_string()).unwrap();

    client
        .send_data(
            tokens::send::CAPTCHA,
            ByteArray::new()
                .write_u32(encoded_captcha.len() as u32)
                .write_bytes(encoded_captcha),
        )
        .await?;

    Ok(())
}
