//! # Codecs
//! 
//! `codecs` is a collection of standard coder and decoder algorithms
//! 
//! ### Available algorithms
//! * Huffman

pub mod huffman;

pub use bincode;
pub use serde::{Deserialize, Serialize};
