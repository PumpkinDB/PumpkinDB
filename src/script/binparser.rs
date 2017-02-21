// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, Needed, ErrorKind};
use script::{Program, ParseError};

pub fn word_tag(i: &[u8]) -> IResult<&[u8], u8> {
    if i.len() < 1 {
        IResult::Incomplete(Needed::Size(1))
    } else if (i[0] & 128 != 128) || i[0] == 128 {
        IResult::Error(ErrorKind::Custom(128))
    } else {
        IResult::Done(&i[0..], i[0] - 128 + 1)
    }
}

pub fn internal_word_tag(i: &[u8]) -> IResult<&[u8], u8> {
    if i.len() < 2 {
        IResult::Incomplete(Needed::Size(2))
    } else if i[0] != 128 || i[1] < 129 {
        IResult::Error(ErrorKind::Custom(128))
    } else {
        IResult::Done(&i[0..], i[1] - 128 + 2)
    }
}

pub fn small_numbers(i: &[u8]) -> IResult<&[u8], &[u8]> {
    if i.len() < 1 {
        IResult::Incomplete(Needed::Size(1))
    } else if i[0] > 10 {
        IResult::Error(ErrorKind::Custom(0))
    } else {
        IResult::Done(&i[1..], &i[0..1])
    }
}

pub fn micro_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 1 {
        IResult::Incomplete(Needed::Size(1))
    } else if i[0] < 11 || i[0] > 110 {
        IResult::Error(ErrorKind::Custom(10))
    } else {
        let size = i[0] as usize;
        if size - 11 > i.len() - 1 {
            IResult::Incomplete(Needed::Size(1 + size))
        } else {
            IResult::Done(&i[0..], size - 10)
        }
    }
}

pub fn byte_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 2 {
        IResult::Incomplete(Needed::Size(2))
    } else if i[0] != 111 {
        IResult::Error(ErrorKind::Custom(111))
    } else {
        let size = i[1] as usize;
        if size > i.len() - 2 {
            IResult::Incomplete(Needed::Size(2 + size))
        } else {
            IResult::Done(&i[0..], size + 2)
        }
    }
}

pub fn small_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 3 {
        IResult::Incomplete(Needed::Size(3))
    } else if i[0] != 112 {
        IResult::Error(ErrorKind::Custom(112))
    } else {
        let size = (i[1] as usize) << 8 | i[2] as usize;
        if size > i.len() - 3 {
            IResult::Incomplete(Needed::Size(3 + size))
        } else {
            IResult::Done(&i[0..], size + 3)
        }
    }
}

pub fn big_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 5 {
        IResult::Incomplete(Needed::Size(5))
    } else if i[0] != 113 {
        IResult::Error(ErrorKind::Custom(113))
    } else {
        let size = (i[1] as usize) << 24 | (i[2] as usize) << 16 | (i[3] as usize) << 8 |
                   (i[4] as usize);
        if size > i.len() - 5 {
            IResult::Incomplete(Needed::Size(5 + size))
        } else {
            IResult::Done(&i[0..], size + 5)
        }
    }
}

fn flatten_program(p: Vec<&[u8]>) -> Vec<u8> {
    let mut vec = Vec::new();
    for item in p {
        vec.extend_from_slice(item);
    }
    vec
}

named!(pub data_size<usize>, alt!(micro_length | byte_length | small_length | big_length));
named!(pub data, alt!(small_numbers | length_bytes!(data_size)));
named!(pub word, length_bytes!(word_tag));
named!(pub internal_word, length_bytes!(internal_word_tag));
named!(pub word_or_internal_word, alt!(internal_word | word));
named!(item, alt!(word | data));
named!(split_code<Vec<u8>>, do_parse!(
                             prog: many0!(item) >>
                                   (flatten_program(prog))));


/// Parse code into a program. This function serves mainly
/// as a binary form validator.
pub fn parse(code: &[u8]) -> Result<Program, ParseError> {
    match split_code(code) {
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
        assert!(parse(vec![0x80, 0x81, b'A'].as_slice()).is_err());
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
    fn test_reserved_numbers() {
        let s = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(parse(&s).unwrap(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

}
