// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

use crate::{tokens, Client, Result, Server};

pub async fn handle(
    client: Arc<Mutex<Client>>,
    _server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let round_code = data.read_i32();
    let is_moving_right = data.read_bool();
    let is_moving_left = data.read_bool();
    let position_x = data.read_u32();
    let position_y = data.read_u32();
    let speed_x = data.read_u16();
    let speed_y = data.read_u16();
    let is_jumping = data.read_bool();
    let jump_img = data.read_i8();
    let portal = data.read_i8();
    let is_angle = !data.is_empty();
    let angle = if is_angle { data.read_i16() } else { -1 };
    let speed_angle = if is_angle { data.read_i16() } else { -1 };
    let loc_1 = if is_angle { data.read_bool() } else { false };

    let mut client = client.lock().await;

    client.position_x = position_x as u64 * 800 / 2700;
    client.position_y = position_y as u64 * 800 / 2700;
    client.speed_x = speed_x;
    client.speed_y = speed_y;
    client.is_jumping = is_jumping;

    if is_moving_right || is_moving_left {
        client.is_moving_right = is_moving_right;
        client.is_moving_left = is_moving_left;
    }

    let mut b = ByteArray::new()
        .write_i32(client.id)
        .write_i32(round_code)
        .write_bool(is_moving_right)
        .write_bool(is_moving_left)
        .write_u32(position_x)
        .write_u32(position_y)
        .write_u16(speed_x)
        .write_u16(speed_y)
        .write_bool(is_jumping)
        .write_i8(jump_img)
        .write_i8(portal);

    if is_angle {
        b = b.write_i16(angle).write_i16(speed_angle).write_bool(loc_1);
    }

    let room = Arc::clone(&client.room.as_ref().unwrap());
    let r = room.lock().await;

    r.send_data_except(client.id, tokens::send::PLAYER_MOVEMENT, b)
        .await?;

    Ok(())
}
