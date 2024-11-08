pub struct PacketBuf {
    vector: Vec<u8>,
    read_index: usize,
}

impl PacketBuf {
    pub fn new() -> PacketBuf {
        PacketBuf {
            vector: Vec::new(),
            read_index: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> PacketBuf {
        PacketBuf {
            vector: Vec::with_capacity(capacity),
            read_index: 0,
        }
    }

    pub fn reset_reading(&mut self) {
        self.read_index;
    }

    pub fn write_i8(&mut self, value: i8) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_index += 1;
        i8::from_be_bytes([
            self.vector[self.read_index -1]
        ])
    }

    pub fn write_u8(&mut self, value: u8) {
        self.vector.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_u8(&mut self) -> u8 {
        self.read_index += 1;
        u8::from_be_bytes([
            self.vector[self.read_index -1]
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
}

#[cfg(test)]
pub mod tests {
    use crate::buffer::PacketBuf;

    #[test]
    pub fn test_buffer() {
        let mut buf = PacketBuf::new();
    }
}