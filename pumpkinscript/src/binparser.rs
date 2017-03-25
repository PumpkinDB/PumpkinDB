// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use nom::{IResult, Needed, ErrorKind};
use super::{Program, ParseError};

pub fn instruction_tag(i: &[u8]) -> IResult<&[u8], u8> {
    if i.len() < 1 {
        IResult::Incomplete(Needed::Size(1))
    } else if (i[0] & 128 != 128) || i[0] == 128 {
        IResult::Error(ErrorKind::Custom(128))
    } else {
        IResult::Done(&i[0..], i[0] - 128 + 1)
    }
}

pub fn internal_instruction_tag(i: &[u8]) -> IResult<&[u8], u8> {
    if i.len() < 2 {
        IResult::Incomplete(Needed::Size(2))
    } else if i[0] != 128 || i[1] < 129 {
        IResult::Error(ErrorKind::Custom(128))
    } else {
        IResult::Done(&i[0..], i[1] - 128 + 2)
    }
}

pub fn micro_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 1 {
        IResult::Incomplete(Needed::Size(1))
    } else if i[0] > 120 {
        IResult::Error(ErrorKind::Custom(120))
    } else {
        let size = i[0] as usize;
        if size > i.len() - 1 {
            IResult::Incomplete(Needed::Size(1 + size))
        } else {
            IResult::Done(&i[0..], size + 1)
        }
    }
}

pub fn byte_length(i: &[u8]) -> IResult<&[u8], usize> {
    if i.len() < 2 {
        IResult::Incomplete(Needed::Size(2))
    } else if i[0] != 121 {
        IResult::Error(ErrorKind::Custom(121))
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
    } else if i[0] != 122 {
        IResult::Error(ErrorKind::Custom(122))
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
    } else if i[0] != 123 {
        IResult::Error(ErrorKind::Custom(123))
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
named!(pub data, length_bytes!(data_size));
named!(pub instruction, length_bytes!(instruction_tag));
named!(pub internal_instruction, length_bytes!(internal_instruction_tag));
named!(pub instruction_or_internal_instruction, alt!(internal_instruction | instruction));
named!(item, alt!(instruction | data));
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
    use textparser::parse as parse_text;
    use binparser::parse;

    #[test]
    fn test() {
        let v = parse_text("0x10 DUP").unwrap();
        assert_eq!(parse(v.as_slice()).unwrap(), parse_text("0x10 DUP").unwrap());
    }

    #[test]
    fn test_internal() {
        // should not be able to parse "internal instructions"
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

}
