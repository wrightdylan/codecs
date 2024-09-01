# A collection of coders and decoders

## Codecs:

- Huffman

## To do:

- Arithmetic
- LZW
- RLE

## Huffman

Huffman is a greedy algorithm used to compress large text files. This is accomplished by building a tree based on the frequency of characters in the text. For more, see [article](https://en.wikipedia.org/wiki/Huffman_coding).

### Implementations
- `easy_encode()` provides a simple interface to encode a string to terminal.
- `encode_to_bitstream()` provides a more useful interface that packages the encoded data with the tree, and can be saved to file.
- `decode_from_bitstream()` reverses the above function.

## License

This project is released under the GNU GPL-3.0 license. Check out the [LICENSE](LICENSE) file for more information.