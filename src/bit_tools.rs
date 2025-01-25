//! ## Bit Vector
//! A simple implementation of a vector with bitwise operation.
pub struct BitVec<'a> {
    pub data: &'a [u8],
    pub byte_idx: usize,
    pub bit_idx: u8,
}

impl<'a> BitVec<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, byte_idx: 0, bit_idx: 0 }
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        if self.byte_idx >= self.data.len() {
            return None;
        }

        let bit = (self.data[self.byte_idx] >> (7 - self.bit_idx)) & 1;
        self.bit_idx += 1;
        if self.bit_idx == 8 {
            self.byte_idx += 1;
            self.bit_idx = 0;
        }

        Some(bit)
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let mut byte: u8 = 0;
        for _ in 0..8 {
            if let Some(bit) = self.read_bit() {
                byte = (byte << 1) | bit;
            } else {
                return None;
            }
        }
        Some(byte)
    }
}