# Wire Protocol

PumpkinDB's wire protocol does not define any data types or package types as
of yet. This is done as PumpkinDB is still in early development so defining
a wire protocol now would be premature.

The current version of the wire protocol only has two data types in addition
to length prefixed byte arrays: words and the integers 0-10.

## Data instruction
 
### Small range integers
* `0u8..10u8` are used to represent 0 to 10;

### Byte arrays
* `<len @ 11u8..110u8> [_;len]` - byte array from 0-99 can have their size indicated by the
first byte subtracted from `11`;
* `<111u8> <len u8> [_; len]` — byte array from 121 to 255 bytes can have their size indicated
in the second byte, followed by that size's number of bytes, with `111u8` as the first byte;
* `<112u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size
indicated in the second and third bytes (u16), followed by that size's number of bytes,
with `112u8` as the first byte;
* `<113u8> <len u32> [_; len]` — byte array from 65536 to 4294967296 bytes can have their
size indicated in the second, third, fourth and fifth bytes (u32), followed by that size's
number of bytes, with `113u8` as the first byte;

### Words

* `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following
byte array of `len & 128u8` length (len without the highest bit set) is considered a word.
Length must be greater than zero.
* `128u8` is reserved as a prefix to be followed by an internal VM's word (not to be accessible
to the end users).

The rest of the tags are reserved.