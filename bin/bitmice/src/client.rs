// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022 AndrielFR <https://github.com/AndrielFR>

use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct Client {
    address: SocketAddr,
    socket: TcpStream,
}

impl Client {
    pub fn new(address: SocketAddr, socket: TcpStream) -> Self {
        Self {
            address,
            socket,
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub async fn handle(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::spawn(async {

        });

        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), std::io::Error> {
        self.socket.shutdown().await
    }
}
