// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022 AndrielFR <https://github.com/AndrielFR>

use std::net::SocketAddr;
use tokio::net::TcpListener;

use super::Client;

#[derive(Default)]
pub struct Server {
    host: String,
    ports: Vec<u16>,
    clients: Vec<Client>,
}

impl Server {
    pub fn new(host: &str, ports: &[u16]) -> Self {
        Self {
            host: host.to_owned(),
            ports: ports.to_owned(),
            ..Default::default()
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (first_port, ports) = self.ports.split_first()
            .expect("no ports found");

        let listener = TcpListener::bind(format!("{}:{}", self.host, first_port)).await?;

        log::info!("server running on ports {:?}", self.ports);

        loop {
            let (socket, address) = listener.accept().await?;

            if !self.check_address(address) {
                log::info!("new connection: {}", socket.peer_addr()?);

                let client = Client::new(address, socket);
                client.handle().await?;
                self.clients.push(client);
            };
        }
    }

    fn check_address(&self, address: SocketAddr) -> bool {
        self.clients.iter().any(|client| client.address() == address)
    }
}
