// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::offset_by_size;

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
}

impl Encodable for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        write_size_header!(self, vec);
        vec.extend_from_slice(&self);
        vec
    }
}

impl Encodable for String {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        write_size_header!(self, vec);
        vec.extend_from_slice(self.as_bytes());
        vec
    }
}

impl<'a> Encodable for &'a str {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        write_size_header!(self, vec);
        vec.extend_from_slice(self.as_bytes());
        vec
    }
}

impl<'a> Encodable for &'a [u8] {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        write_size_header!(self, vec);
        vec.extend_from_slice(self);
        vec
    }
}

macro_rules! peel {
    ($name: ident, $($other: ident,)*) => (tuple! { $($other,)* })
}

macro_rules! tuple_match {
    () => {};
    ($value: expr, $vec: expr, $($name: ident,)*) => (
        let &($(ref $name,)*) = $value;
        $(
           $vec.append(&mut $name.encode());
        )*
    )
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<$($name : Encodable),*> Encodable for ($($name,)*) {
            #[allow(non_snake_case,unused_variables)]
            fn encode(&self) -> Vec<u8> {
               let ($(ref $name,)*) = *self;
               let mut n = 0;
               $(let $name = $name; n += 1;)*
               let mut vec = Vec::with_capacity(n);

               tuple_match!(self, vec, $($name,)*);

               vec
            }
        }
        peel! { $($name,)* }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
         T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, }

pub struct Instruction<'a>(pub &'a str);

impl<'a> Encodable for Instruction<'a> {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.push(self.0.len() as u8 + 0x80);
        vec.extend_from_slice(self.0.as_bytes());
        vec
    }
}

pub struct InstructionRef<'a>(pub &'a str);

impl<'a> Encodable for InstructionRef<'a> {
    fn encode(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        write_size!(self.0.len() + 1, vec);
        vec.push(self.0.len() as u8 + 0x80);
        vec.extend_from_slice(self.0.as_bytes());
        vec
    }
}

pub struct Closure<T : Encodable>(pub T);

impl<T : Encodable> Encodable for Closure<T> {
    fn encode(&self) -> Vec<u8> {
        self.0.encode().encode()
    }
}

#[derive(PartialEq, Debug)]
pub enum Receivable {
    Data(Vec<u8>),
    Instruction(String)
}

impl Encodable for Receivable {
    fn encode(&self) -> Vec<u8> {
        match self {
            &Receivable::Data(ref data) => data.encode(),
            &Receivable::Instruction(ref instruction) =>
                Instruction(&instruction).encode()
        }
    }
}

use std::convert::TryFrom;
use byteorder::{ReadBytesExt, BigEndian};
use std::io::{Cursor, Read};

impl<'a> TryFrom<&'a mut Cursor<Vec<u8>>> for Receivable {

    type Error = ();

    fn try_from(cursor: &'a mut Cursor<Vec<u8>>) -> Result<Self, Self::Error> {
        if cursor.position() as usize == cursor.get_ref().len() {
            return Err(())
        }
        if cursor.get_ref()[cursor.position() as usize] & 0x80 == 0x80 {
            let len = (cursor.read_u8().unwrap() ^ 0x80) as usize;
            let mut vec = vec![0; len];
            let _ = cursor.read_exact(&mut vec).expect("can't read instruction");
            Ok(Receivable::Instruction(String::from_utf8(vec).unwrap()))
        } else {
            let b1 = cursor.read_u8().unwrap();
            let data = if b1 <= 120 {
                let mut vec = vec![0; b1 as usize];
                let _ = cursor.read_exact(&mut vec).expect("can't read nano");
                vec
            } else if b1 == 121u8 {
                let b2 = cursor.read_u8().unwrap();
                let mut vec = vec![0; b2 as usize];
                let _ = cursor.read_exact(&mut vec).expect("can't read micro");
                vec
            } else if b1 == 122u8 {
                let s = cursor.read_u16::<BigEndian>().unwrap();
                let mut vec = vec![0; s as usize];
                let _ = cursor.read_exact(&mut vec).expect("can't read small");
                vec
            } else if b1 == 123u8 {
                let i = cursor.read_u32::<BigEndian>().unwrap();
                let mut vec = vec![0; i as usize];
                let _ = cursor.read_exact(&mut vec).expect("can't read large");
                vec
            } else {
                return Err(());
            };
            Ok(Receivable::Data(data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn instruction() {
        let program = Instruction("DUP").encode();
        let receivable = Receivable::try_from(&mut Cursor::new(program)).unwrap();
        assert_eq!(receivable, Receivable::Instruction(String::from("DUP")));
    }


    #[test]
    fn data_nano() {
        let program = vec![0;120].encode();
        let receivable = Receivable::try_from(&mut Cursor::new(program)).unwrap();
        assert_eq!(receivable, Receivable::Data(vec![0;120]));
    }

    #[test]
    fn data_micro() {
        let program = vec![0;255].encode();
        let receivable = Receivable::try_from(&mut Cursor::new(program)).unwrap();
        assert_eq!(receivable, Receivable::Data(vec![0;255]));
    }

    #[test]
    fn data_small() {
        let program = vec![0;65535].encode();
        let receivable = Receivable::try_from(&mut Cursor::new(program)).unwrap();
        assert_eq!(receivable, Receivable::Data(vec![0;65535]));
    }

    #[test]
    fn data_large() {
        let program = vec![0;100000].encode();
        let receivable = Receivable::try_from(&mut Cursor::new(program)).unwrap();
        assert_eq!(receivable, Receivable::Data(vec![0;100000]));
    }

    #[test]
    fn program() {
        let program = ("hello", Instruction("DUP")).encode();
        let mut cursor = Cursor::new(program);
        let receivable = Receivable::try_from(&mut cursor).unwrap();
        assert_eq!(receivable, Receivable::Data(String::from("hello").into_bytes()));
        let receivable = Receivable::try_from(&mut cursor).unwrap();
        assert_eq!(receivable, Receivable::Instruction(String::from("DUP")));
        assert!(Receivable::try_from(&mut cursor).is_err());
    }


    #[test]
    fn extra_data() {
        let mut program = ("hello", Instruction("DUP")).encode();
        let mut buf = Vec::new();
        buf.append(&mut program);
        buf.extend_from_slice(b"goodbye");
        let mut cursor = Cursor::new(buf);
        let receivable = Receivable::try_from(&mut cursor).unwrap();
        assert_eq!(receivable, Receivable::Data(String::from("hello").into_bytes()));
        let receivable = Receivable::try_from(&mut cursor).unwrap();
        assert_eq!(receivable, Receivable::Instruction(String::from("DUP")));
        let mut res = vec![0; 7];
        let _ = cursor.read_exact(&mut res).unwrap();
        assert_eq!(res, b"goodbye");
    }

    use textparser::parse;

    #[test]
    fn into() {
        let p = (vec![1u8], (Instruction("DUP"), InstructionRef("DUP"))).encode();
        assert_eq!(parse("1 DUP 'DUP").unwrap(), p);
    }

}
