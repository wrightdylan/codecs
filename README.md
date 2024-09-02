# A collection of coders and decoders

## Codecs:

- Huffman

## To do:

- Arithmetic
- LZW
- RLE

## Huffman

Huffman is a greedy algorithm used to compress large text files. This is accomplished by building a tree based on the frequency of characters in the text. For more, see [article](https://en.wikipedia.org/wiki/Huffman_coding). Compression of files averages about 50%, and handles UTF-8 just fine.

Update: `Serde` serialisation works out to be quite large, and it also includes a lot of empty bytes, most likely used as a fixed width header to describe the length of serialised bytes. Preliminary testing using a custom serialisation shows a reduction of the tree information to a 5th of `Serde`'s output. This uses a custom schema as follows:
┌─┬──╌╌──┬─┬──╌╌┄┄┄┄╌╌──┐\
└─┴──╌╌──┴─┴──╌╌┄┄┄┄╌╌──┘\
1 byte: Tree data length in bytes\
n bytes: Tree data\
1 byte: Number of data packing bits\
m bytes: Data (indefinite length)\

This custom serialisation works perfectly for ASCII encoding, or single byte UTF-8, but it breaks multiple byte UTF-8. This can be fixed to account for variable width UTF-8 encoding, however the resulting tree data would probably not be that much smaller than simply sticking to `Serde`, but this is highly dependent on what language is being stored in the tree.

### Implementations
- `easy_encode()` provides a simple interface to encode a string to terminal.
- `encode_to_bitstream()` provides a more useful interface that packages the encoded data with the tree, and can be saved to file.
- `decode_from_bitstream()` reverses the above function.

## License

This project is released under the GNU GPL-3.0 license. Check out the [LICENSE](LICENSE) file for more information.