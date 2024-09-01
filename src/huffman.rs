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
use serde::{Serialize, Deserialize};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct BitStream {
    tree: Node,
    pack: u8,
    data: Vec<u8>,
}

impl BitStream {
    fn new(tree: Node, pack: usize, data: Vec<u8>) -> Self {
        Self {
            tree,
            pack: pack as u8,
            data,
        }
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
    let pack = 8 - encoded.len() % 8;
    encoded.push_str(&"0".repeat(pack));

    let mut data = Vec::new();
    let mut chunk_start = 0;
    while let Some(chunk) = encoded.get(chunk_start..chunk_start + 8) {
        data.push(u8::from_str_radix(chunk, 2).unwrap());
        chunk_start += 8;
    }

    let glob = BitStream::new(tree, pack, data);

    // Serialise struct to binary format
    let output = bincode::serialize(&glob).unwrap();

    Ok(output)
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
    let mut output = String::new();

    // Deserialise binary data to a struct
    let glob: BitStream = match bincode::deserialize(&input) {
        Ok(glob) => glob,
        Err(err) => return Err(anyhow!(err))
    };

    let last_byte = glob.data.len() - 1;
    let mut nodeptr = &glob.tree;
    for (count, byte) in glob.data.iter().enumerate() {
        let end_bit = if count != last_byte { 0 } else { glob.pack };
        for i in (end_bit..8).rev() {
            let bit = (byte >> i) & 1;
            if bit == 0 {
                nodeptr = nodeptr.left.as_ref().unwrap();
            } else {
                nodeptr = nodeptr.right.as_ref().unwrap();
            }
            if let Some(ch) = nodeptr.ch {
                output.push(ch);
                nodeptr = &glob.tree;
            }
        }
    }

    Ok(output)
} 