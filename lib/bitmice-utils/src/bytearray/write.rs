// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022 AndrielFR <https://github.com/AndrielFR>

use bytes::BufMut;

use super::ByteArray;

impl ByteArray {
    pub fn clear(&mut self) {
        self.bytes.clear()
    }

    pub fn write_i8(mut self, byte: i8) -> Self {
        self.bytes.put_i8(byte);

        self
    }

    pub fn write_u8(mut self, byte: u8) -> Self {
        self.bytes.put_u8(byte);

        self
    }

    pub fn write_i16(mut self, short: i16) -> Self {
        self.bytes.put_i16(short);

        self
    }

    pub fn write_u16(mut self, short: u16) -> Self {
        self.bytes.put_u16(short);

        self
    }

    pub fn write_i32(mut self, int: i32) -> Self {
        self.bytes.put_i32(int);

        self
    }

    pub fn write_u32(mut self, int: u32) -> Self {
        self.bytes.put_u32(int);

        self
    }

    pub fn write_i64(mut self, long: i64) -> Self {
        self.bytes.put_i64(long);

        self
    }

    pub fn write_u64(mut self, long: u64) -> Self {
        self.bytes.put_u64(long);

        self
    }

    pub fn write_bool(self, byte: bool) -> Self {
        self.write_u8(byte as u8)
    }

    pub fn write_utf(self, utf: &str) -> Self {
        self.write_u16(utf.len() as u16)
            .write_bytes(utf.bytes().collect::<Vec<u8>>())
    }

    pub fn write_bytes(mut self, bytes: impl Into<ByteArray>) -> Self {
        self.bytes.append(&mut bytes.into().bytes);

        self
    }
}
