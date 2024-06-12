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
    client_: Arc<Mutex<Client>>,
    server: Arc<Mutex<Server>>,
    mut data: ByteArray,
    _packet_id: u8,
) -> Result {
    let community = data.read_utf();
    let mut room_name = data.read_utf();
    let auto_select = data.read_bool();

    let client = client_.lock().await;

    if auto_select || room_name.is_empty() {
        let server = server.lock().await;
        let room = server.get_recommended_room(community.clone()).await;
        let mut r = room.lock().await;

        room_name = r.name.clone();

        drop(client);
        r.add_client(Arc::clone(&client_)).await?;

        let is_new = r.is_new;

        drop(r);
        drop(server);

        let mut client = client_.lock().await;
        client.enter_room(&room_name).await?;
        drop(client);
        crate::client::start_play(Arc::clone(&client_)).await?;

        if is_new {
            room::trigger(Arc::clone(&room)).await?;
        }
    }

    let client = client_.lock().await;
    let room = client.room.clone();
    let room = room.as_ref().unwrap();
    let mut r = room.lock().await;
    if !(room_name == r.name && client.lang == r.lang
        || r.map_type == MapType::Editor
        || room_name.len() > 64)
    {
        drop(client);
        r.add_client(Arc::clone(&client_)).await?;

        let is_new = r.is_new;
        drop(r);

        let mut client = client_.lock().await;
        client
            .enter_room(&format!("{}-{}", community, room_name))
            .await?;
        drop(client);
        crate::client::start_play(Arc::clone(&client_)).await?;

        if is_new {
            room::trigger(Arc::clone(&room)).await?;
        }
    }

    Ok(())
}
