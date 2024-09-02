//! ## Huffman
//! Huffman is a greedy algorithm used to compress large text files. This is
//! accomplished by building a tree based on the frequency of characters in the
//! text. For more, see [article](https://en.wikipedia.org/wiki/Huffman_coding).
//! 
//! ### Implementations
//! 
//! - `easy_encode()` provides a simple interface to encode a string to terminal.
//! - `encode_to_bitstream()` provides a more useful interface that packages the
//! encoded data with the tree, and can be saved to file.
//! - `decode_from_bitstream()` reverses the above function.
use anyhow::{anyhow, Result};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq)]
struct Node {
    ch:    Option<char>,
    left:  Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new_leaf(ch: char) -> Self {
        Self {
            ch: Some(ch),
            left: None,
            right: None,
        }
    }

    fn new_node(left: Box<Node>, right: Box<Node>) -> Self {
        Self {
            ch: None,
            left: Some(left),
            right: Some(right),
        }
    }
}

#[derive(PartialEq, Eq)]
struct Branch {
    node: Box<Node>,
    freq: usize,
}

impl Branch {
    fn new(node: Box<Node>, freq: usize) -> Self {
        Self { node, freq }
    }
}

impl PartialOrd for Branch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.freq.cmp(&self.freq))
    }
}

impl Ord for Branch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct BitBundle<'a> {
    data: &'a [u8],
    byte_idx: usize,
    bit_idx: u8,
}

impl<'a> BitBundle<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, byte_idx: 0, bit_idx: 0 }
    }

    fn read_bit(&mut self) -> Option<u8> {
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

    fn read_byte(&mut self) -> Option<u8> {
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

// Build a Huffman tree and discard frequencies (greatly reduces the size of the tree when serialised)
fn gen_tree(input: &str) -> Node {
    // Count the characters
    let mut char_count: HashMap<char, usize> = HashMap::new();
    for c in input.chars() {
        *char_count.entry(c).or_insert(0) += 1;
    }

    // Populate a min-heap to start building a tree
    let mut tree: BinaryHeap<Branch> = char_count
        .into_iter()
        .map(|(ch, freq)| Branch::new(Box::new(Node::new_leaf(ch)), freq))
        .collect();

    // Build the Huffman tree using greedy algorithm
    while tree.len() > 1 {
        let left = Box::new(tree.pop().unwrap());
        let right = Box::new(tree.pop().unwrap());

        let interior = Branch {
            node: Box::new(Node::new_node(left.node, right.node)),
            freq: left.freq + right.freq,
        };

        tree.push(interior);
    }

    // The root of the tree is the final node left in the heap
    tree.pop().unwrap().node.as_ref().to_owned()
}

fn assign_codes(root: &Node) -> HashMap<char, String> {
    // Generate the codes
    let mut codes = HashMap::new();
    _assign_codes(root, &mut codes, String::new());
    codes
}

// Recursive helper functon to assign codes to characters
fn _assign_codes(node: &Node, codes: &mut HashMap<char, String>, code: String) {
    if let Some(ch) = node.ch {
        codes.insert(ch, code.clone());
    } else {
        if let Some(ref l) = node.left {
            _assign_codes(l, codes, code.clone() + "0");
        }
        if let Some(ref r) = node.right {
            _assign_codes(r, codes, code.clone() + "1");
        }
    }
}

// Convert a String of bits to a vector of bytes
fn bits_to_bytes(bits: String) -> Vec<u8> {
    let mut data = Vec::new();
    let mut chunk_start = 0;
    while let Some(chunk) = bits.get(chunk_start..chunk_start + 8) {
        data.push(u8::from_str_radix(chunk, 2).unwrap());
        chunk_start += 8;
    }

    data
}

// Recursive function to traverse the tree
fn traverse_tree(node: &Node, bit_str: &mut String) {
    if let Some(ch) = node.ch {
        bit_str.push('1');
        bit_str.push_str(&format!("{:08b}", ch.to_string().into_bytes()[0]));
    } else {
        bit_str.push('0');
        traverse_tree(node.left.as_ref().unwrap(), bit_str);
        traverse_tree(node.right.as_ref().unwrap(), bit_str);
    }
}

// Serialise binary tree. This is done via preoder traversal of the tree.
// Preliminary tests show this compresses the tree to a fifth of the original.
fn ser_tree(tree: Node) -> Vec<u8> {
    let mut bit_str = String::new();

    traverse_tree(&tree, &mut bit_str);

    let pack = (8 - bit_str.len() % 8) % 8;
    bit_str.push_str(&"0".repeat(pack));

    bits_to_bytes(bit_str)
}

fn build_tree(bundle: &mut BitBundle) -> Option<Node> {
    if let Some(bit) = bundle.read_bit() {
        if bit == 1 {
            // Leaf node
            if let Some(ch) = bundle.read_byte() {
                return Some(Node::new_leaf(char::from(ch)));
            }
        } else if bundle.byte_idx + 1 != bundle.data.len() {
            // Internal node
            let left = Box::new(build_tree(bundle).unwrap());
            let right = Box::new(build_tree(bundle).unwrap());
            return Some(Node::new_node(left, right));
        }
    }

    None
}

// Restores binary tree from serialisation
fn des_tree(bytes: &[u8]) -> Node {
    let mut bundle = BitBundle::new(bytes);
    build_tree(&mut bundle).unwrap()
}

// Main encoder function
fn encode(input: &str, codes: &HashMap<char, String>) -> String {
    let mut output = String::new();

    for ch in input.chars() {
        let t = codes.get(&ch).unwrap();
        output.push_str(t);
    }

    output
}

/// A fun little function for a quick output showing codes and an encoded
/// string. This function is one way.
/// 
/// ## Example
/// 
/// 
/// ```
/// use codecs::huffman::easy_encode;
/// 
/// let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
/// let (_, output) = easy_encode(input).unwrap();
/// println!("Encoded string: {output}");
/// ```
pub fn easy_encode(input: &str) -> Result<(HashMap<char, String>, String)> {
    if input.is_empty() {
        return Err(anyhow!("Input string is empty."));
    }

    let tree = gen_tree(input);
    let codes = assign_codes(&tree);
    let encoded = encode(input, &codes);

    Ok((codes, encoded))
}

/// Encodes a text and packages it with the tree in a compact binary format for portability.
/// Useful for transmission or archival purposes, and can be decompressed later.
/// 
/// ## Example
/// 
/// 
/// ```
/// use codecs::huffman::encode_to_bitstream;
/// 
/// let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
/// let data = match encode_to_bitstream(&input) {
///     Ok(data) => data,
///     Err(err) => panic!("Something went wrong: {}", err),
/// };
/// 
/// if let Err(err) = fs::write("output.hmc", data) {
///     eprintln!("Error writing to file: {}", err);
/// } else {
///     println!("Data successfully written.");
/// }
/// ```
pub fn encode_to_bitstream(input: &str) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(anyhow!("Input string is empty."));
    }

    let tree = gen_tree(input);
    let codes = assign_codes(&tree);
    let mut encoded = encode(input, &codes);
    let stree = ser_tree(tree);
    let pack = (8 - encoded.len() % 8) % 8;
    encoded.push_str(&"0".repeat(pack));

    // Serialise all data according to schema
    let mut glob = vec![stree.len() as u8];
    glob.extend_from_slice(&stree);
    glob.push(pack as u8);
    glob.extend_from_slice(&bits_to_bytes(encoded));    

    Ok(glob)
}

/// Decompresses a raw binary format and retrieves the tree and encoded data for decoding.
/// 
/// ## Example
/// 
/// 
/// ```
/// use codecs::huffman::decode_from_bitstream;
/// 
/// let file = "output.hmc";
/// let data: Vec<u8> = fs::read(file).expect("File not found.");
/// let output = decode_from_bitstream(&data)?;
/// println!("{output}");
/// ```
pub fn decode_from_bitstream(input: &[u8]) -> Result<String> {
    if input.len() < 4 {
        return Err(anyhow!("Malformed input."));
    }
    let mut output = String::new();

    // Deserialise binary data to variables
    let tree_len = input[0] as usize;
    let tree_bytes = &input[1..(1 + tree_len)];
    let pack = input[1 + tree_len];
    let data = input[(2 + tree_len)..].to_vec();
    if tree_bytes.len() < tree_len {
        return Err(anyhow!("Tree size mismatch."));
    }

    // Decode the data
    let last_byte = data.len() - 1;
    let tree = des_tree(tree_bytes);
    let mut nodeptr = &tree;
    for (count, byte) in data.iter().enumerate() {
        let end_bit = if count != last_byte { 0 } else { pack };
        for i in (end_bit..8).rev() {
            let bit = (byte >> i) & 1;
            if bit == 0 {
                nodeptr = nodeptr.left.as_ref().unwrap();
            } else {
                nodeptr = nodeptr.right.as_ref().unwrap();
            }
            if let Some(ch) = nodeptr.ch {
                output.push(ch);
                nodeptr = &tree;
            }
        }
    }

    Ok(output)
} 