//! ## Bit Vector
//! A simple implementation of a vector with bitwise operation.
//! Second revision for improved memory management, faster performance, and
//! expanded functionality.
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::fmt::{self, Display, Formatter};
use std::ptr::NonNull;

pub struct BitVec {
    ptr: NonNull<u8>,
    cap: usize,      // capacity in bytes
    len: usize,      // length in bits
    byte_idx: usize, // for sequential reading
    bit_idx: u8,     // for sequential reading
}

impl BitVec {
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Converts the vector from bytes to ascii characters when printing
    pub fn as_ascii(&self) -> AsciiWrapper<'_> {
        AsciiWrapper(self)
    }

    /// Generate a new BitVector from an array
    pub fn from(data: &[u8]) -> Self {
        let cap = data.len();
        let len = cap * 8;
        
        // Allocate memory and copy data
        let layout = Layout::array::<u8>(cap).unwrap();
        let ptr = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, cap);
            NonNull::new_unchecked(ptr)
        };
        
        Self {
            ptr,
            cap,
            len,
            byte_idx: 0,
            bit_idx: 0,
        }
    }

    /// Grow the vector by 8 bits
    fn grow(&mut self, min_additional_bytes: usize) {
        let new_cap = if self.cap == 0 {
            min_additional_bytes.max(1)
        } else {
            // Double the capacity, ensuring we have at least min_additional_bytes
            (self.cap * 2).max(self.cap + min_additional_bytes)
        };

        // Create layout for the new allocation
        let layout = Layout::array::<u8>(new_cap).unwrap();
        
        unsafe {
            let new_ptr = if self.cap == 0 {
                alloc(layout)
            } else {
                let old_layout = Layout::array::<u8>(self.cap).unwrap();
                std::alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    old_layout,
                    new_cap
                )
            };

            match NonNull::new(new_ptr as *mut u8) {
                Some(p) => {
                    self.ptr = p;
                    self.cap = new_cap;
                }
                None => handle_alloc_error(layout),
            }
        }
    }

    /// Sets the bit at the desired index
    pub fn set_bit(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        // Ensure we have enough capacity
        if byte_index >= self.cap {
            self.grow(byte_index - self.cap + 1);
        }

        unsafe {
            let byte_ptr = self.ptr.as_ptr().add(byte_index);
            if value {
                *byte_ptr |= 1 << bit_index;
            } else {
                *byte_ptr &= !(1 << bit_index);
            }
        }

        self.len = self.len.max(index + 1);
    }

    /// Returns the bit value at the desired index
    pub fn get_bit(&self, index: usize) -> Option<bool> {
        let byte_index = index / 8;
        let bit_index = index % 8;

        if index >= self.len {
            return None;
        }

        unsafe {
            let byte = *self.ptr.as_ptr().add(byte_index);
            Some((byte & (1 << bit_index)) != 0)
        }
    }

    /// Get the bit index (typically used for reading)
    pub fn get_bit_idx(&self) -> u8 {
        self.bit_idx
    }

    /// Get the byte index (typically used for reading)
    pub fn get_byte_idx(&self) -> usize {
        self.byte_idx
    }

    /// Get the current capacity in bytes
    pub fn len(&self) -> usize {
        self.cap
    }

    /// Get the number of bits stored in the vector (always less than or equal to the capacity)
    pub fn len_bits(&self) -> usize {
        self.len
    }

    /// Removes and returns the last bit in the vector
    pub fn pop_bit(&mut self) -> Option<bool> {
        if self.len == 0 {
            return None;
        }

        // Get the byte containing our bit
        let byte = unsafe { *self.ptr.as_ptr().add(self.byte_idx) };
        
        // Get the bit value
        let bit = (byte >> self.bit_idx) & 1 == 1;
        
        // Update indices
        if self.bit_idx == 0 {
            if self.byte_idx > 0 {
                self.byte_idx -= 1;
                self.bit_idx = 7;
            }
        } else {
            self.bit_idx -= 1;
        }
        
        self.len -= 1;
        Some(bit)
    }

    /// Removes and returns the last byte in the vector regardless of position.
    /// If there is less than 8 bits, then it returns what is available.
    pub fn pop_byte(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        let mut result = 0u8;
        let bits_to_pop = std::cmp::min(8, self.len);
        
        // Safety: ptr is guaranteed to be valid due to NonNull
        unsafe {
            for i in (0..bits_to_pop).rev() {
                let byte_pos = (self.len - 1 - i) / 8;
                let bit_pos = 7 - ((self.len - 1 - i) % 8);
                
                let byte = *self.ptr.as_ptr().add(byte_pos);
                if (byte >> bit_pos) & 1 == 1 {
                    result |= 1 << i;
                }
            }
        }

        self.len -= bits_to_pop;
        Some(result)
    }

    /// Removes and returns the last (complete) byte in the vector
    pub fn pop_full_byte(&mut self) -> Option<u8> {
        if self.len < 8 {
            return None;
        }

        // Can only pop a full byte if we're aligned
        if self.bit_idx != 7 {
            return None;
        }

        let byte = unsafe { *self.ptr.as_ptr().add(self.byte_idx) };
        
        if self.byte_idx > 0 {
            self.byte_idx -= 1;
        } else {
            self.bit_idx = 0;
        }

        self.len -= 8;
        Some(byte)
    }

    /// Pushes a bit to the vector
    pub fn push_bit(&mut self, bit: bool) {
        let byte_offset = self.len / 8;
        let bit_offset = (self.len % 8) as u8;
        
        // Need to allocate a new byte
        if bit_offset == 0 {
            if byte_offset >= self.cap {
                let new_cap = if self.cap == 0 {
                    1 // Start with 1 byte
                } else {
                    self.cap + 1 // Grow by one
                };
                
                let layout = Layout::array::<u8>(self.cap).unwrap();
                let new_layout = Layout::array::<u8>(new_cap).unwrap();
                
                let new_ptr = unsafe {
                    let new_ptr = alloc(new_layout);
                    if new_ptr.is_null() {
                        handle_alloc_error(new_layout);
                    }
                    if self.cap > 0 {
                        std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr, self.cap);
                        dealloc(self.ptr.as_ptr() as *mut u8, layout);
                    }
                    NonNull::new_unchecked(new_ptr)
                };
                
                self.ptr = new_ptr;
                self.cap = new_cap;
            }
            
            // Initialize new byte to 0
            unsafe {
                *self.ptr.as_ptr().add(byte_offset) = 0;
            }
        }
        
        // Set the bit
        unsafe {
            let byte_ptr = self.ptr.as_ptr().add(byte_offset);
            if bit {
                *byte_ptr |= 1 << (7 - bit_offset);
            } else {
                *byte_ptr &= !(1 << (7 - bit_offset));
            }
        }
        
        self.len += 1;
    }

    /// Pushes a byte to the vector
    pub fn push_byte(&mut self, byte: u8) {
        // Check if we need to grow
        if self.cap * 8 == self.len {
            let new_cap = if self.cap == 0 {
                1 // Start with 1 byte
            } else {
                self.cap + 1 // Grow by one
            };
            
            let layout = Layout::array::<u8>(self.cap).unwrap();
            let new_layout = Layout::array::<u8>(new_cap).unwrap();
            
            let new_ptr = unsafe {
                let new_ptr = alloc(new_layout);
                if new_ptr.is_null() {
                    handle_alloc_error(new_layout);
                }
                if self.cap > 0 {
                    std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr, self.cap);
                    dealloc(self.ptr.as_ptr() as *mut u8, layout);
                }
                NonNull::new_unchecked(new_ptr)
            };
            
            self.ptr = new_ptr;
            self.cap = new_cap;
        }
        
        // Push the new byte
        unsafe {
            let byte_offset = self.len / 8;
            *self.ptr.as_ptr().add(byte_offset) = byte;
        }
        
        self.len += 8;
    }
    

    /// Deprecated as the name is too similar to a new function
    #[deprecated(since = "0.1.0", note = "This function is deprecated, please use the seq_read function instead.")]
    pub fn read_bit(&mut self) -> Option<u8> {
        self.seq_read()
    }

    /// This is a sequential read which increments the bit index, not a return of the bit value at a specific index.
    /// For the latter functionality use get_bit().
    pub fn seq_read(&mut self) -> Option<u8> {
        if self.byte_idx >= (self.len + 7) / 8 {
            return None;
        }

        let bit = unsafe {
            let byte = *self.ptr.as_ptr().add(self.byte_idx);
            (byte >> (7 - self.bit_idx)) & 1
        };

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
            if let Some(bit) = self.seq_read() {
                byte = (byte << 1) | bit;
            } else {
                return None;
            }
        }
        Some(byte)
    }

    /// Reset sequential reading position
    pub fn reset_seq_read(&mut self) {
        self.byte_idx = 0;
        self.bit_idx = 0;
    }

    /// Get current reading position in bits
    pub fn get_read_position(&self) -> usize {
        self.byte_idx * 8 + self.bit_idx as usize
    }

    /// Set reading position in bits
    pub fn set_read_position(&mut self, bit_position: usize) -> bool {
        if bit_position >= self.len {
            return false;
        }
        self.byte_idx = bit_position / 8;
        self.bit_idx = (bit_position % 8) as u8;
        true
    }
}

// Implement Drop to properly deallocate memory
impl Drop for BitVec {
    fn drop(&mut self) {
        if self.cap > 0 {
            unsafe {
                let layout = Layout::array::<u8>(self.cap).unwrap();
                dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

// Implement Clone
impl Clone for BitVec {
    fn clone(&self) -> Self {
        let mut new = BitVec::new();
        if self.cap > 0 {
            new.grow(self.cap);
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ptr.as_ptr(),
                    new.ptr.as_ptr(),
                    self.cap
                );
            }
            new.len = self.len;
        }
        new
    }
}

// Implement Display
impl Display for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes = unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), (self.len + 7) / 8)
        };
        
        for (i, &byte) in bytes.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", byte)?;
        }
        
        Ok(())
    }
}

// Wrapper type for ASCII display
pub struct AsciiWrapper<'a>(&'a BitVec);

impl Display for AsciiWrapper<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bitvec = self.0;
        let bytes = unsafe {
            std::slice::from_raw_parts(bitvec.ptr.as_ptr(), (bitvec.len + 7) / 8)
        };
        
        for &byte in bytes {
            write!(f, "{}", byte as char)?;
        }
        
        Ok(())
    }
}