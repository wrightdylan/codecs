# A collection of coders and decoders

## Codecs:

- Huffman

## To do:

- Arithmetic
- LZW
- RLE

## Huffman

Huffman is a greedy algorithm used to compress large text files. This is accomplished by building a tree based on the frequency of characters in the text. For more, see [article](https://en.wikipedia.org/wiki/Huffman_coding). Compression of files averages about 50%, and handles UTF-8 just fine.

Update: `Serde` serialisation works out to be quite large, and it also includes a lot of empty bytes, most likely used as a fixed width header to describe the length of serialised bytes. Preliminary testing using a custom serialisation shows a reduction of the tree information to a 5th of `Serde`'s output. 
This uses a custom schema as follows:
┌───┬──╌╌──┬─┬──╌╌┄┄┄┄╌╌──┐\
└───┴──╌╌──┴─┴──╌╌┄┄┄┄╌╌──┘\
2 or 1-4 bytes: Tree data length either in two bytes or variable width bytes.\
n bytes: Tree data\
1 byte: Number of data packing bits\
m bytes: Data (indefinite length)\

This custom serialisation works perfectly for ASCII encoding, or single byte UTF-8, but it breaks multiple byte UTF-8. This can be fixed to account for variable width UTF-8 encoding, however the resulting tree data would probably not be that much smaller than simply sticking to `Serde`, but this is highly dependent on what language is being stored in the tree.

Update 2:
The original tree data length of 1 byte was enough for standard Roman characters and some 2-byte Unicode languages, but 3-byte Unicode presented some problems even with a short sentence due to overflows. A fixed with of 2 bytes (1 word) was used instead. There did not appear to be much point in having a variable width header, but an additional byte for Romance languages will not make much of a difference, and it is unlikely more than 65,535 bytes will be needed unless a very large text in Japanese, for example, that uses all known characters in the language will be compressed. Ultimately, compression is still very good with Romance languages, but it suffers to varying degrees with others.

Update 3:
Implementing variable width headers was just far too tempting. This is now one to four bytes, which will allow 28 bits of tree length information, but if you need 268,435,456 bytes for your tree, you're probably doing something very wrong. See the [section below](#variable-width-encoding) for more details on this encoding.

Update 4:
Fixed width or variable width headers can now be selected as a feature. The default is 2-byte fixed width, or use the `vwe_header` feature for the option.

### Implementations
- `easy_encode()` provides a simple interface to encode a string to terminal.
- `encode_to_bitstream()` provides a more useful interface that packages the encoded data with the tree, and can be saved to file.
- `decode_from_bitstream()` reverses the above function.

## Variable width encoding
Two new functions deal with encoding/decoding variable width headers. These are internal to the codec library, and are not intended for use externally. The first takes an unsigned int, ideally usize, and checks that it's less than the  maximum value of a 28-bit number. Numbers below 128 can be stored in a single byte where the most significant bit is 0, and the remaining bits are for data. Larger numbers will use an encoded first byte, and the remainder will be normal bytes. The first byte will have a 1 for each trailing byte, and a 0 separator. The table below shows how bytes are encoded for their size.

| Range | First byte |
|:-|:-|
| 0 - 127                 | 0XXX_XXXX |
| 128 - 16_383            | 10XX_XXXX |
| 16_384 - 2_097_151      | 110X_XXXX |
| 2_097_152 - 268,435,455 | 1110_XXXX |


## License

This project is released under the GNU GPL-3.0 license. Check out the [LICENSE](LICENSE) file for more information.