// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, ErrorKind};
use nom::{is_hex_digit, is_space, is_digit};

use num_bigint::BigUint;
use core::str::FromStr;
use std::str;

use script::{Program, ParseError};

fn prefix_word(word: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.push(word.len() as u8 | 128u8);
    vec.extend_from_slice(word);
    vec
}

#[inline]
fn hex_digit(v: u8) -> u8 {
    match v {
        0x61u8...0x66u8 => v - 32 - 0x41 + 10,
        0x41u8...0x46u8 => v - 0x41 + 10,
        _ => v - 48,
    }
}

macro_rules! write_size {
    ($vec : expr, $size : expr) => {
      match $size {
        0...120 => $vec.push($size as u8),
        121...255 => {
            $vec.push(121u8);
            $vec.push($size as u8);
        }
        256...65535 => {
            $vec.push(122u8);
            $vec.push(($size >> 8) as u8);
            $vec.push($size as u8);
        }
        65536...4294967296 => {
            $vec.push(123u8);
            $vec.push(($size >> 24) as u8);
            $vec.push(($size >> 16) as u8);
            $vec.push(($size >> 8) as u8);
            $vec.push($size as u8);
        }
        _ => unreachable!()
      }
    };
}


fn bin(bin: &[u8]) -> Vec<u8> {
    let mut bin_ = Vec::new();
    for i in 0..bin.len() - 1 {
        if i % 2 != 0 {
            continue;
        }
        bin_.push((hex_digit(bin[i]) << 4) | hex_digit(bin[i + 1]));
    }
    let mut vec = Vec::new();
    let size = bin_.len();
    write_size!(vec, size);
    vec.extend_from_slice(bin_.as_slice());
    vec
}

fn string_to_vec(s: &[u8]) -> Vec<u8> {
    let mut bin = Vec::new();
    let size = s.len();
    write_size!(bin, size);
    bin.extend_from_slice(s);
    bin
}

fn sized_vec(s: Vec<u8>) -> Vec<u8> {
    let mut vec = Vec::new();
    let size = s.len();
    write_size!(vec, size);
    vec.extend_from_slice(s.as_slice());
    vec

}

fn is_word_char(s: u8) -> bool {
    (s >= b'a' && s <= b'z') || (s >= b'A' && s <= b'Z') || (s >= b'0' && s <= b'9') ||
    s == b'_' || s == b':' || s == b'-' || s == b'=' ||
    s == b'!' || s == b'#' || s == b'$' || s == b'%' || s == b'@' || s == b'?' ||
    s == b'/'
}


fn is_crlf(s: u8) -> bool {
    s == 10 || s == 13
}

fn flatten_program(p: Vec<Vec<u8>>) -> Vec<u8> {
    let mut vec = Vec::new();
    for mut item in p {
        vec.append(&mut item);
    }
    vec
}

fn delim_or_end(i: &[u8]) -> IResult<&[u8], ()> {
    if i.len() == 0 || (i.len() >= 1 && (i[0] == b' ' || i[0] == b']')) {
        return IResult::Done(&i[0..], ())
    } else {
        IResult::Error(ErrorKind::Custom(0))
    }
}

fn eof(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    if i.len() == 0 {
        return IResult::Done(&i[0..], Vec::new())
    } else {
        IResult::Error(ErrorKind::Custom(1))
    }
}

named!(uint<Vec<u8>>, do_parse!(
                     biguint: take_while1!(is_digit)      >>
                              delim_or_end                >>
                              (sized_vec(BigUint::from_str(str::from_utf8(biguint).unwrap())
                                         .unwrap().to_bytes_be()))));
named!(word<Vec<u8>>, do_parse!(
                        word: take_while1!(is_word_char)  >>
                              (prefix_word(word))));
named!(wordref<Vec<u8>>, do_parse!(tag!(b"'") >> w: word >> (sized_vec(w))));
named!(binary<Vec<u8>>, do_parse!(
                              tag!(b"0x")                 >>
                         hex: take_while1!(is_hex_digit)  >>
                              (bin(hex))
));
named!(string<Vec<u8>>,  alt!(do_parse!(tag!(b"\"\"") >> (vec![0])) |
                         do_parse!(
                         str: delimited!(char!('"'), is_not!("\""), char!('"')) >>
                              (string_to_vec(str)))));
named!(code<Vec<u8>>, do_parse!(
                         prog: delimited!(char!('['), ws!(program), char!(']')) >>
                               (sized_vec(prog))));
named!(item<Vec<u8>>, alt!(binary | string | uint | code | wordref | word));
named!(program<Vec<u8>>, alt!(do_parse!(
                               take_while!(is_space)                        >>
                            v: eof                                          >>
                               (v))
                              | do_parse!(
                               take_while!(is_space)                        >>
                         item: separated_list!(take_while!(is_space), item) >>
                               (flatten_program(item)))));
named!(pub programs<Vec<Vec<u8>>>, do_parse!(
                         item: separated_list!(take_while!(is_crlf), program)   >>
                               (item)));

/// Parses human-readable PumpkinScript
///
/// The format is simple, it is a sequence of space-separated tokens,
/// with binaries represented as:
///
/// * `0x<hexadecimal>` (hexadecimal form)
/// * `"STRING"` (string form, no quoted characters support yet)
/// * `integer` (integer form, will convert to a big endian big integer)
/// * `'word` (word in a binary form)
///
/// The rest of the instructions considered to be words.
///
/// One additional piece of syntax is code included within square
/// brackets: `[DUP]`. This means that the parser will take the code inside,
/// compile it to the binary form and add as a data push. This is useful for
/// words like EVAL
///
/// # Example
///
/// ```norun
/// parse("0xABCD DUP DROP DROP")
/// ```
///
/// It's especially useful for testing but there is a chance that there will be
/// a "suboptimal" protocol that allows to converse with PumpkinDB over telnet
pub fn parse(script: &str) -> Result<Program, ParseError> {
    match program(script.as_bytes()) {
        IResult::Done(_, x) => Ok(x),
        IResult::Incomplete(_) => Err(ParseError::Incomplete),
        IResult::Error(ErrorKind::Custom(code)) => Err(ParseError::Err(code)),
        _ => Err(ParseError::UnknownErr),
    }
}

#[cfg(test)]
mod tests {
    use script::textparser::{parse, programs};
    use num_bigint::BigUint;
    use core::str::FromStr;

    #[test]
    fn test_empty() {
        let script = parse("").unwrap();
        assert_eq!(script, vec![]);
        let script = parse("  ").unwrap();
        assert_eq!(script, vec![]);
    }

    #[test]
    fn test_wordref() {
        let script = parse("'HELLO").unwrap();
        assert_eq!(script, vec![0x06, 0x85, b'H', b'E', b'L', b'L', b'O']);
    }

    #[test]
    fn test_one() {
        let script = parse("0xAABB").unwrap();
        assert_eq!(script, vec![2, 0xaa,0xbb]);
        let script = parse("HELLO").unwrap();
        assert_eq!(script, vec![0x85, b'H', b'E', b'L', b'L', b'O']);
    }

    #[test]
    fn test_uint() {
        let script = parse("1234567890").unwrap();
        let mut bytes = BigUint::from_str("1234567890").unwrap().to_bytes_be();
        let mut sized = Vec::new();
        sized.push(4);
        sized.append(&mut bytes);
        assert_eq!(script, sized);
    }

    #[test]
    fn test_many_uint() {
        let script = parse("1 2 3").unwrap();

        let mut vec = Vec::new();

        let mut bytes = BigUint::from_str("1").unwrap().to_bytes_be();
        vec.push(1);
        vec.append(&mut bytes);

        let mut bytes = BigUint::from_str("2").unwrap().to_bytes_be();
        vec.push(1);
        vec.append(&mut bytes);

        let mut bytes = BigUint::from_str("3").unwrap().to_bytes_be();
        vec.push(1);
        vec.append(&mut bytes);

        assert_eq!(script, vec);
    }

    #[test]
    fn test_uint_at_the_end_of_code() {
        let script = parse("[1]").unwrap();
        assert_eq!(script, parse("[0x01]").unwrap());
    }


    #[test]
    fn test_number_prefixed_word() {
        let script = parse("2DUP").unwrap();
        assert_eq!(script, b"\x842DUP");
    }

    #[test]
    fn test_extra_spaces() {
        let script = parse(" 0xAABB  \"Hi\" ").unwrap();
        assert_eq!(script, vec![2, 0xaa,0xbb, 2, b'H', b'i']);
    }

    #[test]
    fn test() {
        let script = parse("0xAABB DUP 0xFF00CC \"Hello\"").unwrap();

        assert_eq!(script, vec![0x02, 0xAA, 0xBB, 0x83, b'D', b'U', b'P',
                                0x03, 0xFF, 0x00, 0xCC, 0x05, b'H', b'e', b'l', b'l', b'o']);
    }


    #[test]
    fn test_empty_string() {
        let script = parse("\"\"").unwrap();

        assert_eq!(script, vec![0]);
    }

    #[test]
    fn test_code() {
        let script = parse("[DUP]").unwrap();
        let script_spaced = parse("[ DUP ]").unwrap();
        assert_eq!(script, vec![4, 0x83, b'D', b'U', b'P']);
        assert_eq!(script_spaced, vec![4, 0x83, b'D', b'U', b'P']);
    }

    #[test]
    fn test_programs() {
        let str = "SOMETHING : BURP DURP\nBURP : DURP";
        let (_, mut progs) = programs(str.as_bytes()).unwrap();
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("BURP : DURP").unwrap());
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("SOMETHING : BURP DURP").unwrap());

        let str = "SOMETHING : BURP DURP\nBURP : DURP";
        let (_, mut progs) = programs(str.as_bytes()).unwrap();
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("BURP : DURP").unwrap());
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("SOMETHING : BURP DURP").unwrap());
    }

}
