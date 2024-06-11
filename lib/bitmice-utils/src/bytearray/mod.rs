// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2022-2024 AndrielFR <https://github.com/AndrielFR>

mod read;
mod write;

#[derive(Clone, Default)]
pub struct ByteArray {
    pub(crate) bytes: Vec<u8>,
}

impl ByteArray {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn with(bytes: Vec<u8>) -> Self {
        Self::new().write_bytes(bytes)
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap()
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.bytes
    }
}

impl Into<ByteArray> for &[u8] {
    fn into(self) -> ByteArray {
        ByteArray {
            bytes: self.to_owned(),
        }
    }
}

impl Into<ByteArray> for Vec<u8> {
    fn into(self) -> ByteArray {
        ByteArray { bytes: self }
    }
}

impl std::fmt::Debug for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut unicode_data = String::new();
        for byte in self.as_bytes() {
            unicode_data.push_str(
                (*byte as char)
                    .escape_unicode()
                    .collect::<String>()
                    .as_str(),
            );
        }

        write!(f, "ByteArray [{}]", unicode_data)
    }
}

#[cfg(test)]
mod tests {
    use super::ByteArray;

    #[test]
    fn clear_and_is_empty() {
        let mut bytearray = ByteArray::default().write_i8(-1);
        assert_eq!(bytearray.is_empty(), false);

        bytearray.clear();
        assert_eq!(bytearray.is_empty(), true);
    }

    #[test]
    fn write_and_read_i8() {
        let mut bytearray = ByteArray::default().write_i8(-1);
        assert_eq!(bytearray.len(), 1);

        assert_eq!(bytearray.read_i8(), -1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_u8() {
        let mut bytearray = ByteArray::default().write_u8(1);
        assert_eq!(bytearray.len(), 1);

        assert_eq!(bytearray.read_u8(), 1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_i16() {
        let mut bytearray = ByteArray::default().write_i16(-1);
        assert_eq!(bytearray.len(), 2);

        assert_eq!(bytearray.read_i16(), -1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_u16() {
        let mut bytearray = ByteArray::default().write_u16(1);
        assert_eq!(bytearray.len(), 2);

        assert_eq!(bytearray.read_u16(), 1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_i32() {
        let mut bytearray = ByteArray::default().write_i32(-1);
        assert_eq!(bytearray.len(), 4);

        assert_eq!(bytearray.read_i32(), -1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_u32() {
        let mut bytearray = ByteArray::default().write_u32(1);
        assert_eq!(bytearray.len(), 4);

        assert_eq!(bytearray.read_u32(), 1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_i64() {
        let mut bytearray = ByteArray::default().write_i64(-1);
        assert_eq!(bytearray.len(), 8);

        assert_eq!(bytearray.read_i64(), -1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_u64() {
        let mut bytearray = ByteArray::default().write_u64(1);
        assert_eq!(bytearray.len(), 8);

        assert_eq!(bytearray.read_u64(), 1);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_bool() {
        let mut bytearray = ByteArray::default().write_bool(true);
        assert_eq!(bytearray.len(), 1);

        assert_eq!(bytearray.read_bool(), true);
        assert_eq!(bytearray.len(), 0);
    }

    #[test]
    fn write_and_read_utf() {
        let mut bytearray = ByteArray::default().write_utf("BitMice");
        assert_eq!(bytearray.len(), 9); // String::len() + 2

        assert_eq!(bytearray.read_utf(), "BitMice");
        assert_eq!(bytearray.len(), 0);
    }
}
