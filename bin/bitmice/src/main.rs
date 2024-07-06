// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod client;
mod room;
mod server;
mod tokens;

use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex};

use client::Client;
use room::Room;
use server::Server;

pub type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let ports = &[11801, 12801, 13801, 14801];
    let server = Server::new(
        567865443,    // auth key
        "WKvjvHsJiT", //ckey
        818,          // version
        &[
            // login keys
            5798205, 2147483648, 16384, 128, 5798205, 2147483648, 16384, 806984, 5798205,
            2147483648, 16384,
        ],
        &[
            // packet keys
            13, 16, 42, 55, 40, 23, 19, 43, 11, 55, 87, 74, 116, 105, 114, 77, 117, 77, 97, 93,
        ],
    );
    /* let server = Server::new(
        0,
        "yAdByj", //ckey
        616,      // version
        &[],
        &[],
    ); */
    let server = Arc::new(Mutex::new(server));

    for port in ports.clone() {
        let s = Arc::clone(&server);

        tokio::spawn(async move {
            loop {
                let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", port))
                    .await
                    .expect("error binding a port");

                if let Ok((socket, address)) = listener.accept().await {
                    let (reader, writer) = socket.into_split();
                    let reader = Arc::new(Mutex::new(reader));
                    let writer = Arc::new(Mutex::new(writer));

                    let client = Client::new(
                        address,
                        Arc::clone(&reader),
                        Arc::clone(&writer),
                        Arc::clone(&s),
                    );
                    server::handle_client(client, reader, writer).await;
                }
            }
        });
    }

    log::info!("server running on ports {:?}", ports);

    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
