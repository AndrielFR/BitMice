// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use std::sync::Arc;

use crate::{
    room::{self, MapType, Room},
    Client, Result, Server,
};
use bitmice_utils::ByteArray;
use tokio::sync::Mutex;

pub async fn handle(
    client: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let community = data.read_utf();
    let mut room_name = data.read_utf();
    let auto_select = data.read_bool();

    let c = client.lock().await;

    if auto_select || room_name.is_empty() {
        let s = server.lock().await;
        let room = s.get_recommended_room(community.clone()).await;
        let mut r = room.lock().await;

        room_name = r.name.clone();

        drop(c);
        r.add_client(Arc::clone(&client)).await?;

        let is_new = r.is_new;

        drop(r);
        drop(s);

        let mut c = client.lock().await;
        c.enter_room(&room_name).await?;
        drop(c);
        crate::client::start_play(Arc::clone(&client)).await?;

        if is_new {
            room::trigger(Arc::clone(&room)).await?;
        }
    }

    let c = client.lock().await;
    let room = c.room.clone();
    let room = room.as_ref().unwrap();
    let mut r = room.lock().await;
    if !(room_name == r.name && c.lang == r.lang
        || r.map_type == MapType::Editor
        || room_name.len() > 64)
    {
        drop(c);
        r.add_client(Arc::clone(&client)).await?;

        let is_new = r.is_new;
        drop(r);

        let mut c = client.lock().await;
        c
            .enter_room(&format!("{}-{}", community, room_name))
            .await?;
        drop(c);
        crate::client::start_play(Arc::clone(&client)).await?;

        if is_new {
            room::trigger(Arc::clone(&room)).await?;
        }
    }

    Ok(())
}
