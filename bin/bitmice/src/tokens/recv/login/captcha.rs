// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{tokens, Client, Result, Server};
use bitmice_utils::{generate_captcha, generate_captcha_image, ByteArray};
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
    let mut pixel_count = 0;

    let mut p = ByteArray::new();
    for row in 0..height {
        for col in 0..width {
            let c = captcha.get_pixel(col, row).0[2];
            if c != 0 {
                p = p.write_i32(c as i32);
                pixel_count += 1;
            }
        }
    }

    client
        .send_data(
            tokens::send::CAPTCHA,
            ByteArray::new()
                .write_i32(0)
                .write_u16(width as u16)
                .write_u16(height as u16)
                .write_u16(pixel_count - 1)
                .write_bytes(p),
        )
        .await?;

    Ok(())
}
