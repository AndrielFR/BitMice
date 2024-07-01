// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

use bytes::Buf;

use super::ByteArray;

impl ByteArray {
    pub fn offset(&mut self, start: usize, end: usize) -> Self {
        Self {
            bytes: self.bytes[start..end].to_vec(),
        }
    }

    pub fn read(&mut self, length: usize) -> Self {
        Self {
            bytes: self.bytes[..length].to_vec(),
        }
    }

    pub fn read_i8(&mut self) -> i8 {
        let mut bytes_slice = self.bytes.as_slice();
        let byte = bytes_slice.get_i8();
        self.bytes = bytes_slice.to_owned();

        byte
    }

    pub fn read_u8(&mut self) -> u8 {
        let mut bytes_slice = self.bytes.as_slice();
        let byte = bytes_slice.get_u8();
        self.bytes = bytes_slice.to_owned();

        byte
    }

    pub fn read_i16(&mut self) -> i16 {
        let mut bytes_slice = self.bytes.as_slice();
        let short = bytes_slice.get_i16();
        self.bytes = bytes_slice.to_owned();

        short
    }

    pub fn read_u16(&mut self) -> u16 {
        let mut bytes_slice = self.bytes.as_slice();
        let short = bytes_slice.get_u16();
        self.bytes = bytes_slice.to_owned();

        short
    }

    pub fn read_i32(&mut self) -> i32 {
        let mut bytes_slice = self.bytes.as_slice();
        let int = bytes_slice.get_i32();
        self.bytes = bytes_slice.to_owned();

        int
    }

    pub fn read_u32(&mut self) -> u32 {
        let mut bytes_slice = self.bytes.as_slice();
        let int = bytes_slice.get_u32();
        self.bytes = bytes_slice.to_owned();

        int
    }

    pub fn read_i64(&mut self) -> i64 {
        let mut bytes_slice = self.bytes.as_slice();
        let long = bytes_slice.get_i64();
        self.bytes = bytes_slice.to_owned();

        long
    }

    pub fn read_u64(&mut self) -> u64 {
        let mut bytes_slice = self.bytes.as_slice();
        let long = bytes_slice.get_u64();
        self.bytes = bytes_slice.to_owned();

        long
    }


    pub fn read_i128(&mut self) -> i128 {
        let mut bytes_slice = self.bytes.as_slice();
        let long = bytes_slice.get_i128();
        self.bytes = bytes_slice.to_owned();

        long
    }

    pub fn read_u128(&mut self) -> u128 {
        let mut bytes_slice = self.bytes.as_slice();
        let long = bytes_slice.get_u128();
        self.bytes = bytes_slice.to_owned();

        long
    }

    pub fn read_bool(&mut self) -> bool {
        !(self.read_u8() == 0)
    }

    pub fn read_utf(&mut self) -> String {
        let length = self.read_u16();

        let mut utf_bytes = Vec::new();
        (0..length).for_each(|_| utf_bytes.push(self.read_u8()));

        String::from_utf8(utf_bytes).unwrap()
    }
}
