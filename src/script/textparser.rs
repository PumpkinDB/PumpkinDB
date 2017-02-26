// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, ErrorKind};
use nom::{is_hex_digit, multispace, is_digit};

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
        0...99 => $vec.push($size as u8 + 11),
        100...255 => {
            $vec.push(111u8);
            $vec.push($size as u8);
        }
        256...65535 => {
            $vec.push(112u8);
            $vec.push(($size >> 8) as u8);
            $vec.push($size as u8);
        }
        65536...4294967296 => {
            $vec.push(113u8);
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
    let s = unsafe { String::from_utf8_unchecked(s.to_vec()) }
            .replace("\\\"","\"")
            .replace("\\n","\n");
    let mut bin = Vec::new();
    let size = s.len();
    write_size!(bin, size);
    bin.extend_from_slice(s.as_bytes());
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
    s == b'/' || s == b'<' || s == b'>'
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

fn is_multispace(s: u8) -> bool {
    s == b'\n' || s == b'\r' || s == b'\t' || s == b' '
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
                         str: delimited!(char!('"'), escaped!(is_not!("\"\\"), '\\', one_of!("\"n\\")), char!('"')) >>
                              (string_to_vec(str)))));
named!(comment<Vec<u8>>, do_parse!(delimited!(char!('('), is_not!(")"), char!(')')) >> (vec![])));
named!(item<Vec<u8>>, alt!(comment | binary | string | uint | wrap | wordref | word));

fn unwrap_word(mut word: Vec<u8>) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.extend_from_slice(b"`");
    vec.append(&mut word);
    vec
}

fn rewrap(prog: Vec<u8>) -> Vec<u8> {
    let mut program = &prog[..];
    let mut vec = Vec::new();
    let mut acc = Vec::new();
    let mut counter = 0;

    while program.len() > 0 {
        if let IResult::Done(rest, unwrap) = bin_unwrap(program) {
            if acc.len() > 0 {
                vec.append(&mut sized_vec(acc.clone()));
                acc.clear();
                counter += 1;
            }
            vec.extend_from_slice(&unwrap[1..]);
            vec.extend_from_slice(b"\x01\x01");
            vec.append(&mut prefix_word(b"WRAP"));

            counter += 1;
            program = rest;
        } else if let IResult::Done(rest, data) = super::binparser::data(program) {
            acc.extend_from_slice(data);
            program = rest;
        } else if let IResult::Done(rest, word) = super::binparser::word(program) {
            acc.extend_from_slice(word);
            program = rest;
        } else {
            panic!("invalid data {:?}", &program);
        }
    }
    if acc.len() > 0 {
        counter += 1;
        vec.append(&mut sized_vec(acc.clone()));
        acc.clear();
    }
    for _ in 0..counter - 1 {
        vec.append(&mut prefix_word(b"CONCAT"));
    }
    if counter == 0 {
        sized_vec(vec)
    } else {
        vec
    }
}

use super::binparser::word_tag;
named!(bin_word<Vec<u8>>, do_parse!(v: length_bytes!(word_tag) >> (Vec::from(v))));

named!(bin_unwrap<Vec<u8>>, do_parse!(
                              tag!(b"`")                   >>
                        word: alt!(bin_word | bin_unwrap)  >>
                              (unwrap_word(word))));

named!(unwrap<Vec<u8>>, do_parse!(
                              tag!(b"`")                 >>
                        word: alt!(word | unwrap)        >>
                              (unwrap_word(word))));
named!(wrap<Vec<u8>>, do_parse!(
                         prog: delimited!(char!('['), ws!(wrapped_program), char!(']')) >>
                               (rewrap(prog))));
named!(wrapped_item<Vec<u8>>, alt!(item | unwrap));
named!(wrapped_program<Vec<u8>>, alt!(do_parse!(
                               take_while!(is_multispace)                        >>
                            v: eof                                               >>
                               (v))
                              | do_parse!(
                               take_while!(is_multispace)                        >>
                         item: separated_list!(multispace, wrapped_item)         >>
                               take_while!(is_multispace)                        >>
                               (flatten_program(item)))));

named!(program<Vec<u8>>, alt!(do_parse!(
                               take_while!(is_multispace)                        >>
                            v: eof                                               >>
                               (v))
                              | do_parse!(
                               take_while!(is_multispace)                        >>
                         item: separated_list!(multispace, item)                 >>
                               take_while!(is_multispace)                        >>
                               (flatten_program(item)))));

named!(pub programs<Vec<Vec<u8>>>, do_parse!(
                         item: separated_list!(tag!(b"."), program)              >>
                               (item)));


/// Parses human-readable PumpkinScript
///
/// The format is simple, it is a sequence of space-separated tokens,
/// with binaries represented as:
///
/// * `0x<hexadecimal>` (hexadecimal form)
/// * `"STRING"` (string form, newline and double quotes can be escaped with `\`)
/// * `integer` (integer form, will convert to a big endian big integer)
/// * `'word` (word in a binary form)
///
/// The rest of the instructions considered to be words.
///
/// One additional piece of syntax is code included within square
/// brackets: `[DUP]`. This means that the parser will take the code inside,
/// compile it to the binary form and add as a data push. This is useful for
/// words like EVAL. Inside of this syntax, you can use so-called "unwrapping"
/// syntax that can embed a value of a word into this code:
///
/// ```norun
/// PumpkinDB> 1 'a SET [`a] 'b SET 2 'a SET b EVAL
/// 0x01
/// ```
///
/// It is also possible to unwrap multiple levels:
///
/// ```norun
/// PumpkinDB> "A" 'a SET [[2 ``a DUP] EVAL] 'b SET "B" 'a SET b EVAL
/// 0x02 "A" "A"
/// ```
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
        IResult::Done(rest, x) => {
            if rest.len() == 0 {
                Ok(x)
            } else {
                Err(ParseError::Superfluous(Vec::from(rest)))
            }
        },
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
        let empty : Vec<u8> = vec![];
        assert_eq!(script, empty);
        let script = parse("  ").unwrap();
        assert_eq!(script, empty);
    }

    #[test]
    fn multiline() {
        let script_multiline = parse("\nHELP [\n\
        DROP] \n\
        1").unwrap();
        let script = parse("HELP [DROP] 1").unwrap();
        assert_eq!(script, script_multiline);
    }

    #[test]
    fn test_comment() {
        let script = parse("1 (hello) 2").unwrap();
        assert_eq!(script, parse("1 2").unwrap());
    }

    #[test]
    fn test_multiline_comment() {
        let script = parse("1 (hel\nlo) 2").unwrap();
        assert_eq!(script, parse("1 2").unwrap());
    }

    #[test]
    fn superfluous() {
        assert!(parse("HELP [DROP]]").is_err());
    }

    #[test]
    fn test_wordref() {
        let script = parse("'HELLO").unwrap();
        assert_eq!(script, vec![0x11, 0x85, b'H', b'E', b'L', b'L', b'O']);
    }

    #[test]
    fn test_one() {
        let script = parse("0xAABB").unwrap();
        assert_eq!(script, vec![13, 0xaa,0xbb]);
        let script = parse("HELLO").unwrap();
        assert_eq!(script, vec![0x85, b'H', b'E', b'L', b'L', b'O']);
    }

    #[test]
    fn test_uint() {
        let script = parse("1234567890").unwrap();
        let mut bytes = BigUint::from_str("1234567890").unwrap().to_bytes_be();
        let mut sized = Vec::new();
        sized.push(15);
        sized.append(&mut bytes);
        assert_eq!(script, sized);
    }

    #[test]
    fn test_many_uint() {
        let script = parse("1 2 3").unwrap();

        let mut vec = Vec::new();

        let mut bytes = BigUint::from_str("1").unwrap().to_bytes_be();
        vec.push(12);
        vec.append(&mut bytes);

        let mut bytes = BigUint::from_str("2").unwrap().to_bytes_be();
        vec.push(12);
        vec.append(&mut bytes);

        let mut bytes = BigUint::from_str("3").unwrap().to_bytes_be();
        vec.push(12);
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
        assert_eq!(script, vec![13, 0xaa,0xbb, 13, b'H', b'i']);
    }

    #[test]
    fn test() {
        let script = parse("0xAABB DUP 0xFF00CC \"Hello\"").unwrap();

        assert_eq!(script, vec![13, 0xAA, 0xBB, 0x83, b'D', b'U', b'P',
                                14, 0xFF, 0x00, 0xCC, 16, b'H', b'e', b'l', b'l', b'o']);
    }


    #[test]
    fn test_empty_string() {
        let script = parse("\"\"").unwrap();

        assert_eq!(script, vec![0]);
    }

    #[test]
    fn test_string_escaping() {
        let script = parse(r#""\"1\"""#).unwrap();
        assert_eq!(script, vec![3, b'"', b'1', b'"']);
        let script = parse(r#""\n""#).unwrap();
        assert_eq!(script, vec![1, b'\n']);
    }

    #[test]
    fn test_wrap() {
        let script = parse("[DUP]").unwrap();
        let script_spaced = parse("[ DUP ]").unwrap();
        assert_eq!(script, vec![15, 0x83, b'D', b'U', b'P']);
        assert_eq!(script_spaced, vec![15, 0x83, b'D', b'U', b'P']);
    }

    #[test]
    fn test_empty_wrap() {
        let script = parse("[]").unwrap();
        assert_eq!(script, vec![0]);
    }

    #[test]
    fn test_programs() {
        let str = "SOMETHING : BURP DURP.\nBURP : DURP";
        let (_, mut progs) = programs(str.as_bytes()).unwrap();
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("BURP : DURP").unwrap());
        assert_eq!(Vec::from(progs.pop().unwrap()), parse("SOMETHING : BURP DURP").unwrap());
    }


    #[test]
    fn unwrapping() {
        assert_eq!(parse("[`val DUP]").unwrap(), parse("val 1 WRAP [DUP] CONCAT").unwrap());
        assert_eq!(parse("[`val]").unwrap(), parse("val 1 WRAP").unwrap());
        assert_eq!(parse("[1 `val DUP]").unwrap(), parse("[1] val 1 WRAP [DUP] CONCAT CONCAT").unwrap());
        assert_eq!(parse("[1 `val DUP `val]").unwrap(), parse("[1] val 1 WRAP [DUP] val 1 WRAP CONCAT CONCAT CONCAT").unwrap());
        assert_eq!(parse("[1 `val]").unwrap(), parse("[1] val 1 WRAP CONCAT").unwrap());
    }

    #[test]
    fn nested_unwrapping() {
        assert_eq!(parse("[[``val DUP]]").unwrap(), parse("val 1 WRAP [1 WRAP [DUP] CONCAT] CONCAT").unwrap());
        assert_eq!(parse("[[2 ``val DUP]]").unwrap(), parse("[[2]] val 1 WRAP [1 WRAP [DUP] CONCAT CONCAT] CONCAT CONCAT").unwrap());
    }

}
