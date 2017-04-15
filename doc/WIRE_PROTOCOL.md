# Wire Protocol

This document describes the design of the PumpkinDB wire protocol. It is intrinsically linked to [PumpkinScript](script/README.md) as all
communication with PumpkinDB is done via PumpkinScript scripts execution.

Since core PumpkinScript, having only binary type, is an essentially untyped language (with typing capabilities planned to be built on top of it), the wire protocol is extremely simple as it needs to be able to convey only two types of information: binaries and instructions (instructions).

## Data

* `<len @ 0..120u8> [_;len]` — byte arrays of up to 120 bytes can have their size indicated
in the first byte, followed by that size's number of bytes;
* `<121u8> <len u8> [_; len]` — byte array from 121 to 255 bytes can have their size indicated
in the second byte, followed by that size's number of bytes, with `121u8` as the first byte;
* `<122u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size
indicated in the second and third bytes (u16), followed by that size's number of bytes,
with `122u8` as the first byte;
* `<123u8> <len u32> [_; len]` — byte array from 65536 to 4294967296 bytes can have their
size indicated in the second, third, fourth and fifth bytes (u32), followed by that size's
number of bytes, with `123u8` as the first byte.

## Instructions

* `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following
byte array of `len & 128u8` length (len without the highest bit set) is considered a instruction.
Length must be greater than zero;
* `128u8` is reserved as a prefix to be followed by an internal Scheduler's instruction (not to be
 accessible to the end users).

The rest of tags (`124u8` to `127u8`) are reserved for future use.
