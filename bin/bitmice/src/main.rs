// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022 AndrielFR <https://github.com/AndrielFR>

mod client;
pub mod server;

use client::Client;
use server::Server;

#[tokio::main]
async fn main() {
    let host = "0.0.0.0";
    let ports = &[1201, 1202];

    let server = Server::new(host, ports);
    server.run().await
        .expect("failed to start the server");
}
