// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{collections::HashMap, sync::Arc, time::UNIX_EPOCH};

use crate::{room, tokens, Client, Result, Server};
use bitmice_utils::{language_id, ByteArray, OldData};
use tokio::sync::Mutex;

pub async fn handle(
    client_: Arc<Mutex<Client>>,
    server_: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut identity = data.read_utf();
    let password = data.read_utf();
    let _url = data.read_utf();
    let mut start_room = data.read_utf();
    let _result_key = data.read_i32();

    let mut client = client_.lock().await;
    let mut server = server_.lock().await;

    if identity.is_empty() && password.is_empty() || identity.len() < 3 {
        client // invalid account
            .send_data(
                tokens::send::LOGIN_RESULT,
                ByteArray::new()
                    .write_i8(2)
                    .write_utf(&identity)
                    .write_utf(&password),
            )
            .await?;
        return Ok(());
    } else if password.is_empty() {
        if !identity.starts_with("+") {
            identity = format!("*{}", identity);
        }

        start_room = format!("\x03[Tutorial] {}", identity);

        client.priv_level = 0;
        client.time_played = UNIX_EPOCH.elapsed().unwrap().as_millis() as i64;
        client.is_guest = true;
    }

    client.name = identity;

    client.id = server.new_player_id();
    drop(server);

    // player identification
    let mut p = ByteArray::new();
    let mut perms = Vec::new();
    let priv_authorization = HashMap::from([
        (0, -1),
        (1, -1),
        (2, -1),
        (3, -1),
        (4, -1),
        (5, 13),
        (6, 11),
        (7, 5),
        (8, 5),
        (9, 10),
    ]);

    for (priv_, auth) in priv_authorization {
        if client.priv_level >= priv_ {
            perms.push(auth);
        }
    }

    if client.priv_level >= 7 {
        perms.push(1);
        perms.push(3);
    }

    if client.priv_level >= 9 {
        perms.push(10);
    }

    for perm in perms.iter() {
        p = p.write_i8(*perm)
    }

    let data = ByteArray::new()
        .write_i32(client.id)
        .write_utf(&client.name)
        .write_i32(client.time_played as i32)
        .write_i8(language_id(&client.lang))
        .write_i32(client.id)
        .write_bool(true)
        .write_i8(perms.len() as i8)
        .write_bytes(p)
        .write_bool(client.priv_level >= 9)
        .write_i16(255)
        .write_i16(0);
    client
        .send_data(tokens::send::PLAYER_IDENTIFICATION, data)
        .await?;

    // player login
    let old_data = vec![
        OldData::String(client.full_name()),
        OldData::Integer(client.id),
        OldData::Byte(client.priv_level),
        OldData::Byte(30),
        OldData::Bool(client.is_souris()),
        OldData::Integer(0),
    ];
    client
        .send_old_data(tokens::old::send::LOGIN, old_data)
        .await?;

    // guest login
    if client.is_souris() {
        client
            .send_data(
                tokens::send::LOGIN_SOURIS,
                ByteArray::new().write_i8(1).write_i8(10),
            )
            .await?;
        client
            .send_data(
                tokens::send::LOGIN_SOURIS,
                ByteArray::new().write_i8(2).write_i8(5),
            )
            .await?;
        client
            .send_data(
                tokens::send::LOGIN_SOURIS,
                ByteArray::new().write_i8(3).write_i8(15),
            )
            .await?;
        client
            .send_data(
                tokens::send::LOGIN_SOURIS,
                ByteArray::new().write_i8(4).write_u8(200),
            )
            .await?;
    }

    // enter room
    client.enter_room(&start_room).await?;

    // add player to room
    let room = Arc::clone(&client.room.as_ref().unwrap());
    let mut r = room.lock().await;
    drop(client);
    r.add_client(Arc::clone(&client_)).await?;
    let is_new = r.is_new;
    drop(r);
    crate::client::start_play(Arc::clone(&client_)).await?;

    if is_new {
        room::trigger(Arc::clone(&room)).await?;
    }

    // send anchors
    let mut client = client_.lock().await;
    client
        .send_old_data(tokens::old::send::ANCHORS, vec![])
        .await?;

    Ok(())
}
