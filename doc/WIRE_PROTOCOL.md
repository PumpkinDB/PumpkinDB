# Wire Protocol

This document describes the design of the PumpkinDB wire protocol.
It's current design is focused around simplicity, with most data
not tagged by types but simply broken up into byte arrays.

## Data Push Instructions

* `<len @ 0..120u8> [_;len]` — byte arrays of up to 120 bytes can have their size indicated
in the first byte, followed by that size's number of bytes;
* `<121u8> <len u8> [_; len]` — byte array from 121 to 255 bytes can have their size indicated
in the second byte, followed by that size's number of bytes, with `121u8` as the first byte;
* `<122u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size
indicated in the second and third bytes (u16), followed by that size's number of bytes
with `122u8` as the first byte;
* `<122u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size
indicated in the second and third bytes (u16), followed by that size's number of bytes,
with `122u8` as the first byte;
* `<123u8> <len u32> [_; len]` — byte array from 65536 to 4294967296 bytes can have their
size indicated in the second, third, fourth and fifth bytes (u32), followed by that size's
number of bytes, with `123u8` as the first byte.

## Words

* `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following
byte array of `len & 128u8` length (len without the highest bit set) is considered a word.
Length must be greater than zero;
* `128u8` is reserved as a prefix to be followed by an internal Scheduler's word (not to be
 accessible to the end users).

The rest of tags (`124u8` to `127u8`) are reserved for future use.