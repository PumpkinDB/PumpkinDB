// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, ErrorKind};
use script::{Program, ParseError};
use nom::{is_hex_digit, is_space};

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
        _ => unimplemented!()
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

fn is_word_char(s: u8) -> bool {
    (s >= b'a' && s <= b'z') || (s >= b'A' && s <= b'Z') || (s >= b'0' && s <= b'9') ||
        s == b'_' || s == b':' || s == b'-'
}

fn program_to_vec(p: Vec<Vec<u8>>) -> Vec<u8> {
    let mut vec = Vec::new();
    let s = p.iter().fold(0, |s, i| i.len() + s);
    write_size!(vec, s);
    for mut item in p {
        vec.append(&mut item);
    }
    vec
}

named!(word<Vec<u8>>, do_parse!(
                        word: take_while1!(is_word_char)  >>
                              (prefix_word(word))));
named!(binary<Vec<u8>>, do_parse!(
                              tag!(b"0x")                 >>
                         hex: take_while1!(is_hex_digit)  >>
                              (bin(hex))
));
named!(string<Vec<u8>>, do_parse!(
                         str: delimited!(char!('"'), is_not!("\""), char!('"')) >>
                              (string_to_vec(str))));
named!(code<Vec<u8>>, do_parse!(
                         prog: delimited!(char!('['), ws!(program), char!(']')) >>
                               (program_to_vec(prog))));
named!(item<Vec<u8>>, alt!(binary | string | code | word));
named!(program<Vec<Vec<u8>>>, do_parse!(
                               take_while!(is_space)                            >>
                         item: separated_list!(take_while!(is_space), item)     >>
                               take_while!(is_space)                            >>
                               (item)));

/// Parses human-readable PumpkinScript
///
/// The format is simple, it is a sequence of space-separated tokens,
/// which binaries represented `0x<hexadecimal>` or `"STRING"`
/// (no quoted characters support yet)
/// and the rest of the instructions considered to be words
///
/// # Example
///
/// ```
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
    use script::execute;
    use script::textparser::parse;
    use futures::Future;

    #[test]
    fn test_one() {
        let mut script = parse("0xAABB").unwrap();
        assert_eq!(script.len(), 1);
        assert_eq!(script.pop(), Some(vec![2, 0xaa,0xbb]));
        let mut script = parse("HELLO").unwrap();
        assert_eq!(script.len(), 1);
        assert_eq!(script.pop(), Some(vec![0x85, b'H', b'E', b'L', b'L', b'O']));
    }

    #[test]
    fn test_extra_spaces() {
        let mut script = parse(" 0xAABB  \"Hi\" ").unwrap();
        assert_eq!(script.len(), 2);
        assert_eq!(script.pop(), Some(vec![2, b'H', b'i']));
        assert_eq!(script.pop(), Some(vec![2, 0xaa,0xbb]));
    }

    #[test]
    fn test() {
        let script = parse("0xAABB DUP 0xFF00CC \"Hello\"").unwrap();
        let aabb = [0x02, 0xAA, 0xBB];
        let dup = [0x83, b'D', b'U', b'P'];
        let ff00cc = [0x03, 0xFF, 0x00, 0xCC];
        let hello = [0x05, b'H', b'e', b'l', b'l', b'o'];
        let mut vec: Vec<&[u8]> = Vec::new();
        vec.push(&aabb);
        vec.push(&dup);
        vec.push(&ff00cc);
        vec.push(&hello);
        assert_eq!(script, vec);

        let (mut stack, _) = execute(Vec::new(), script).wait().unwrap();

        stack.pop();
        stack.pop();
        stack.pop();
        stack.pop();
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_code() {
        let script = parse("[DUP]").unwrap();
        let script_spaced = parse("[ DUP ]").unwrap();
        let dup = [4, 0x83, b'D', b'U', b'P'];
        let mut vec: Vec<&[u8]> = Vec::new();
        vec.push(&dup);
        assert_eq!(script, vec);
        assert_eq!(script_spaced, vec);
    }

}
