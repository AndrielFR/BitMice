// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod client;
mod room;
mod server;
mod tokens;

use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use client::Client;
use room::Room;
use server::Server;

pub type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

// extern crate bitmice_macros;

#[tokio::main]
async fn main() {
    env_logger::init();

    let ports = &[11801, 12801, 13801, 14801];

    let server = Server::default();

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
                    let reader = Arc::new(tokio::sync::Mutex::new(reader));
                    let writer = Arc::new(tokio::sync::Mutex::new(writer));

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

    loop {}
}
