use anyhow::Result;
use codecs::huffman::{encode_to_bitstream, decode_from_bitstream};
use std::fs;

fn main() -> Result<()> {
    if let Ok(input) = fs::read_to_string("examples/book.txt") {
        let data = match encode_to_bitstream(&input) {
            Ok(data) => data,
            Err(err) => panic!("Something went wrong: {}", err),
        };

        let out_file = "output.hmc";
        if let Err(err) = fs::write(out_file, data) {
            eprintln!("Error writing to file: {}", err);
        } else {
            println!("Data successfully written to {}", out_file);
        }
    } else {
        eprintln!("Error reading file");
    }

    let file = "output.hmc";
    let data: Vec<u8> = fs::read(file).expect("File not found.");
    let output = decode_from_bitstream(&data)?;
    println!("Recovered text: {}", output);

    Ok(())
}