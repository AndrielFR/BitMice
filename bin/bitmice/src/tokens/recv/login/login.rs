// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::{collections::HashMap, sync::Arc, time::UNIX_EPOCH};

use crate::{room, tokens, Client, Result, Server};
use bitmice_utils::{generate_captcha, language_id, ByteArray, OldData};
use rand::{thread_rng, Rng};
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let mut identity = data.read_utf();
    let password = data.read_utf();
    let _url = data.read_utf();
    let mut start_room = data.read_utf();
    let _result_key = data.read_i32();

    let mut s = server.lock().await;

    if identity.is_empty() && password.is_empty() || identity.len() < 3 {
        let mut c = client.lock().await;
        c // invalid account
            .send_data(
                tokens::send::LOGIN_RESULT,
                ByteArray::new()
                    .write_i8(2)
                    .write_utf(&identity)
                    .write_utf(&password),
            )
            .await?;
        return Ok(());
    } else if s.get_player(identity.clone()).await.is_some() {
        let mut c = client.lock().await;

        if password.is_empty(){
            let random_name = generate_captcha(thread_rng().gen_range(6..16)).to_lowercase();
            c // already connected, choose another name
                .send_data(
                    tokens::send::LOGIN_RESULT,
                    ByteArray::new()
                        .write_i8(3)
                        .write_utf(&random_name)
                        .write_utf(""),
                )
                .await?;
        } else {
            c // already connected
                .send_data(
                    tokens::send::LOGIN_RESULT,
                    ByteArray::new()
                        .write_i8(1)
                        .write_utf(&identity)
                        .write_utf(&password),
                )
                .await?;
        }

        return Ok(());
    } else if password.is_empty() {
        if !identity.starts_with("+") {
            identity = format!("*{}", identity);
        }

        start_room = format!("\x03[Tutorial] {}", identity);

        let mut c = client.lock().await;
        c.priv_level = 0;
        c.time_played = UNIX_EPOCH.elapsed().unwrap().as_millis() as i64;
        c.is_guest = true;
    }

    let mut c = client.lock().await;
    c.name = identity;

    c.id = s.new_player_id();
    drop(c);
    drop(s);

    identification(Arc::clone(&client)).await?;
    login(Arc::clone(&client)).await?;

    // enter room
    let mut c = client.lock().await;
    c.enter_room(&start_room).await?;
    drop(c);

    add_to_room(Arc::clone(&client)).await?;

    // send anchors
    let mut c = client.lock().await;
    c
        .send_old_data(tokens::old::send::ANCHORS, vec![])
        .await?;

    Ok(())
}

async fn identification(client: Arc<Mutex<Client>>) -> Result {
    let mut client = client.lock().await;

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
        .await
}


async fn login(client: Arc<Mutex<Client>>) -> Result {
    let mut client = client.lock().await;

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
     Ok(())
}

async fn add_to_room(client: Arc<Mutex<Client>>) -> Result {
    let c = client.lock().await;
    let room = Arc::clone(&c.room.as_ref().unwrap());
    let mut r = room.lock().await;
    drop(c);

    r.add_client(Arc::clone(&client)).await?;
    let is_new = r.is_new;
    drop(r);
    crate::client::start_play(Arc::clone(&client)).await?;

    if is_new {
        room::trigger(Arc::clone(&room)).await?;
    }

    Ok(())
}
