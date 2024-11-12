use std::ops::{BitAnd, BitOr, Not, Shl};

#[derive(Debug)]
pub struct PacketBuf {
    vector: Vec<u8>,
    read_index: usize,
}

impl Default for PacketBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketBuf {
    pub fn new() -> PacketBuf {
        PacketBuf {
            vector: vec![],
            read_index: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> PacketBuf {
        PacketBuf {
            vector: vec![0; capacity],
            read_index: 0,
        }
    }

    pub fn as_array(&self) -> &[u8] {
        self.vector.as_ref()
    }

    pub fn as_mut_array(&mut self) -> &mut [u8] {
        self.vector.as_mut()
    }

    pub fn resize(&mut self, new_length: usize) {
        if new_length <= self.vector.len() {
            self.vector.truncate(new_length);
        } else {
            self.vector.resize(new_length, 0);
        }
    }

    pub fn length(&self) -> usize {
        self.vector.len()
    }

    pub fn capacity(&self) -> usize {
        self.vector.capacity()
    }

    pub fn reset_reading(&mut self) {
        self.read_index = 0;
    }

    pub fn write_all(&mut self, buf: &PacketBuf) {
        self.vector.extend_from_slice(&buf.vector);
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        self.vector.extend_from_slice(slice);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_index += 1;
        i8::from_be_bytes([
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_u8(&mut self, value: u8) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u8(&mut self) -> u8 {
        self.read_index += 1;
        u8::from_be_bytes([
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_i16(&mut self, value: i16) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i16(&mut self) -> i16 {
        self.read_index += 2;
        i16::from_be_bytes([
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_u16(&mut self, value: u16) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u16(&mut self) -> u16 {
        self.read_index += 2;
        u16::from_be_bytes([
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_i32(&mut self, value: i32) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i32(&mut self) -> i32 {
        self.read_index += 4;
        i32::from_be_bytes([
            self.vector[self.read_index - 4],
            self.vector[self.read_index - 3],
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_u32(&mut self, value: u32) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u32(&mut self) -> u32 {
        self.read_index += 4;
        u32::from_be_bytes([
            self.vector[self.read_index - 4],
            self.vector[self.read_index - 3],
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ])
    }

    pub fn int64_slice(&self) -> [u8; 8] {
        [
            self.vector[self.read_index - 8],
            self.vector[self.read_index - 7],
            self.vector[self.read_index - 6],
            self.vector[self.read_index - 5],
            self.vector[self.read_index - 4],
            self.vector[self.read_index - 3],
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ]
    }

    pub fn write_i64(&mut self, value: i64) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i64(&mut self) -> i64 {
        self.read_index += 8;
        i64::from_be_bytes(self.int64_slice())
    }

    pub fn write_u64(&mut self, value: u64) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u64(&mut self) -> u64 {
        self.read_index += 8;
        u64::from_be_bytes(self.int64_slice())
    }

    pub fn int128_slice(&self) -> [u8; 16] {
        [
            self.vector[self.read_index - 16],
            self.vector[self.read_index - 15],
            self.vector[self.read_index - 14],
            self.vector[self.read_index - 13],
            self.vector[self.read_index - 12],
            self.vector[self.read_index - 11],
            self.vector[self.read_index - 10],
            self.vector[self.read_index - 9],
            self.vector[self.read_index - 8],
            self.vector[self.read_index - 7],
            self.vector[self.read_index - 6],
            self.vector[self.read_index - 5],
            self.vector[self.read_index - 4],
            self.vector[self.read_index - 3],
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ]
    }

    pub fn write_i128(&mut self, value: i128) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i128(&mut self) -> i128 {
        self.read_index += 16;
        i128::from_be_bytes(self.int128_slice())
    }

    pub fn write_u128(&mut self, value: u128) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u128(&mut self) -> u128 {
        self.read_index += 16;
        u128::from_be_bytes(self.int128_slice())
    }

    const SEGMENT_BITS: i64 = 0x7F;
    const CONTINUE_BIT: i64 = 0x80;

    pub fn read_var_int(&mut self) -> i64 {
        let mut value: i64 = 0;
        let mut position = 0;

        loop {
            let current_byte = self.read_u8();

            value |= (current_byte as i64 & (PacketBuf::SEGMENT_BITS)) << position;

            if (current_byte & (PacketBuf::CONTINUE_BIT as u8)) == 0 { break; }

            position += 7;
        }

        value
    }

    pub fn write_var_int(&mut self, mut value: i64) {
        let mut position = 0;

        loop {
            if (value & !PacketBuf::SEGMENT_BITS) == 0 {
                self.write_u8(value as u8);
                return;
            }

            self.write_u8((((value & 0xFF) & PacketBuf::SEGMENT_BITS) | PacketBuf::CONTINUE_BIT) as u8);

            value = ((value as u64) >> 7) as i64;
        }
    }

    pub fn write_f32(&mut self, value: f32) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_f32(&mut self) -> f32 {
        self.read_index += 4;
        f32::from_be_bytes([
            self.vector[self.read_index - 4],
            self.vector[self.read_index - 3],
            self.vector[self.read_index - 2],
            self.vector[self.read_index - 1]
        ])
    }

    pub fn write_f64(&mut self, value: f64) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_f64(&mut self) -> f64 {
        self.read_index += 8;
        f64::from_be_bytes(self.int64_slice())
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.vector.push(value as u8);
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_index += 1;
        self.vector[self.read_index - 1] == 1
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_var_int(value.len() as i64);
        self.vector.extend_from_slice(value.as_bytes());
    }

    pub fn read_string(&mut self) -> String {
        let length = self.read_var_int();
        let mut str = String::new();
        for _ in 0..length {
            self.read_index += 1;
            str.push(self.vector[self.read_index - 1] as char);
        };
        str
    }
}

#[cfg(test)]
pub mod tests {
    use crate::buffer::PacketBuf;

    #[test]
    pub fn test_buffer() {
        let mut buf = PacketBuf::new();
        buf.write_var_int(328033232455);
        println!("{:?}", buf.vector);
        println!("{}", buf.read_var_int());
    }
}