// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Compose
//!
//! This module is intended for composing PumpkinScript code in Rust
//! clients
//!
use super::offset_by_size;

/// # Item
///
/// Represents a [Program](type.Program.html) item
pub enum Item<'a> {
    Data(&'a [u8]),
    Instruction(&'a str),
    InstructionRef(&'a str),
}


impl<'a> Into<Vec<u8>> for Item<'a> {

    fn into(self) -> Vec<u8> {
        let mut vec : Vec<u8> = Vec::new();
        match self {
            Item::Instruction(instruction) => {
                vec.push(instruction.len() as u8 + 0x80);
                vec.extend_from_slice(instruction.as_bytes());
            }
            Item::InstructionRef(instruction) => {
                write_size!(instruction.len() + 1, vec);
                vec.push(instruction.len() as u8 + 0x80);
                vec.extend_from_slice(instruction.as_bytes());
            }
            Item::Data(data) => {
                write_size_header!(data, vec);
                vec.extend_from_slice(data)
            }
        }
        vec
    }

}


/// # Program
///
/// Represents an entire program
pub struct Program<'a>(pub Vec<Item<'a>>);


impl<'a> Into<Vec<u8>> for Program<'a> {

    fn into(self) -> Vec<u8> {
        let mut vec = Vec::new();
        for item in self.0 {
            vec.append(&mut item.into());
        }
        vec
    }

}

#[cfg(test)]
mod tests {

    use super::Program;
    use super::Item::*;
    use script::parse;

    #[test]
    fn into() {
        let p : Vec<u8> = Program(vec![Data(&vec![1]), Instruction("DUP"), InstructionRef("DUP")]).into();
        assert_eq!(parse("1 DUP 'DUP").unwrap(), p);

    }

}