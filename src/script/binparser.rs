// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, ErrorKind};
use script::{Program, ParseError};

use nom::{be_u8, be_u16, be_u32};

named!(mint_length<usize>,
    peek!(do_parse!(
        one_of!(&[0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8]) >>
        (1))));

named!(n_word_length<usize>,
    peek!(do_parse!(
        tag: peek!(take!(1)) >>
        l: cond_reduce!((tag[0] & 128 == 128) && !(tag[0] == 128), take!(1)) >>
        ((l[0] - 128 + 1) as usize))));

named!(n_internal_word_length<usize>,
    peek!(do_parse!(
        tag!(&[128u8][..]) >>
        length: be_u8 >>
        ((length - 128 + 2) as usize))));

named!(n_small_length<usize>,
    peek!(do_parse!(
        tag!(&[111u8][..]) >>
        length: be_u8 >>
        ((length + 2) as usize))));

named!(n_medium_length<usize>,
    peek!(do_parse!(
        tag!(&[112u8][..]) >>
        length: be_u16 >>
        ((length + 3) as usize))));

named!(n_large_length<usize>,
    peek!(do_parse!(
        tag!(&[113u8][..]) >>
        length: be_u32 >>
        ((length + 5) as usize))));

named!(word_size<usize>, alt!(n_word_length | n_internal_word_length));
named!(pub data_size<usize>, alt!(n_small_length | n_medium_length | n_large_length | mint_length));
named!(pub data, alt!(length_bytes!(data_size)));
named!(pub word_or_internal_word, length_bytes!(word_size));
named!(pub word, length_bytes!(n_word_length));

named!(program<Vec<u8>>,
    do_parse!(
        program: many0!(alt!(length_bytes!(data_size) | length_bytes!(n_word_length))) >>
        ({
            let mut vec = Vec::new();
            for item in program {
                vec.extend_from_slice(item);
            }
            vec
        })
));


/// Parse code into a program. This function serves mainly
/// as a binary form validator.
pub fn parse(code: &[u8]) -> Result<Program, ParseError> {
    match program(code) {
        IResult::Done(_, x) => Ok(x),
        IResult::Incomplete(_) => Err(ParseError::Incomplete),
        IResult::Error(ErrorKind::Custom(code)) => Err(ParseError::Err(code)),
        _ => Err(ParseError::UnknownErr),
    }
}

#[cfg(test)]
mod tests {
    use script::parse as parse_text;
    use script::binparser::parse;

    #[test]
    fn test_basic() {
        let v = parse_text("0x10 DUP").unwrap();
        assert_eq!(parse(v.as_slice()).unwrap(), parse_text("0x10 DUP").unwrap());
    }

    #[test]
    fn test_internal() {
        // should not be able to parse "internal words"
        let ok: &[u8] = &[];
        assert_eq!(parse(vec![0x80, 0x81, b'A'].as_slice()).unwrap(), ok);
    }

    #[test]
    fn test_byte() {
        let mut byte_sized_sequence: String = "0x".to_owned();
        for _ in 1..200 {
            byte_sized_sequence.push_str("AA");
        }
        let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
        assert_eq!(parse(v.as_slice()).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
    }

    #[test]
    fn test_small() {
        let mut byte_sized_sequence: String = "0x".to_owned();
        for _ in 1..300 {
            byte_sized_sequence.push_str("AA");
        }
        let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
        assert_eq!(parse(v.as_slice()).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
    }

    #[test]
    fn test_big() {
        let mut byte_sized_sequence: String = "0x".to_owned();
        for _ in 1..70000 {
            byte_sized_sequence.push_str("AA");
        }
        let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
        assert_eq!(parse(v.as_slice()).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
    }

    #[test]
    fn test_mint() {
        let v: &[u8] = &vec![0,1,2,3,4,5,6,7,8,9,10];
        assert_eq!(parse(&v).unwrap(), v);
    }
}
