// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # PumpkinScript
//!
//! PumpkinScript is a minimalistic concatenative, stack-based language inspired
//! by Forth.
//!
//! It is used in PumpkinDB to operate a low-level database "virtual machine" —
//! to manipulate, record and retrieve data.
//!
//! This is an ultimate gateway to flexibility in how PumpkinDB can operate, what
//! formats can it support, etc.
//!
//! # Reasoning
//!
//! Why is it important?
//!
//! In previous incarnations (or, rather, inspirations) of PumpkinDB much more rigid structures,
//! formats and encoding were established as a prerequisite for using it, unnecessarily limiting
//! the applicability and appeal of the technology and ideas behind it. For example, one had to buy
//! into [ELF](https://rfc.eventsourcing.com/spec:1/ELF), UUID-based event identification and
//! [HLC-based](https://rfc.eventsourcing.com/spec:6/HLC) timestamps.
//!
//! So it was deemed to be important to lift this kind of restrictions in PumpkinDB. But how do we
//! support all the formats without knowing what they are?
//!
//! What if there was a way to describe how data should be processed, for example,
//! for indexing — in a compact, unambiguous and composable form? Or even for recording data
//! itself?
//! Well, that's where the idea to use something like a Forth-like script was born.
//!
//! Instead of devising custom protocols for talking to PumpkinDB, the protocol of communication has
//! become a pipeline to a script executor.
//!
//! So, for example, a command/events set can be recorded with something like this (not an actual
//! script, below is pseudocode):
//!
//! ```forth
//! <command id> <command payload> JOURNAL <event id> <event payload> JOURNAL
//! ```
//!
//! This offers us enormous extension and flexibility capabilities. To name a few:
//!
//! * Low-level imperative querying (as a foundation for declarative queries)
//! * Indexing filters
//! * Subscription filters
//!
//! # Usage
//!
//! The main entry point for executing PumpkinScript is [`Env`](struct.vm.html) via
//! the [`Executor`](trait.Executor.html) trait.
//!
//! # Examples
//!
//! ```
//! let (mut stack, _) = script::execute(Vec::new(), "\"Hello\" DROP").wait().unwrap();
//! assert_eq!(stack.pop(), None);
//! ```


use futures::future;
use futures::{Future, BoxFuture};

/// `word!` macro is used to define a known (embedded) word, its signature (if applicable)
/// and representation
macro_rules! word {
    ($name : ident,
    ($($input : ident),* => $($output : ident),*),
    $ident : expr) =>
    (
     word!($name, $ident);
    );
    ($name : ident,
    $ident : expr) =>
    (
     const $name : &'static[u8] = $ident;
    )
}

// Built-in words
// TODO: the list of built-in words is far from completion

// Stack words

/// `DROP` removes an item from the top of the stack
word!(DROP, (a => ), b"\x84DROP");
/// `DUP` duplicates an item at the top of the stack
word!(DUP, (a => a, a), b"\x83DUP");
/// `SWAP` swaps the order of the two topmost items on the stack
word!(SWAP, (a, b => b, a), b"\x84SWAP");
/// `ROT` moves third item from the top to the top
word!(ROT, (a, b, c  => b, c, a), b"\x83ROT");
/// `OVER` copies the second topmost item to the top of the stack
word!(OVER, (a, b => a, b, a), b"\x84OVER");

// Byte arrays

/// `CONCAT` takes two topmost items and concatenates them, and
/// pushes result to the top of the stack
word!(CONCAT, (a, b => c), b"\x86CONCAT");

// Control flow

/// `EVAL` takes the topmost item and evaluates it as a PumpkinScript
/// program on the current stack
word!(EVAL, b"\x84EVAL");

/// `Instruction` is a type alias for `Vec<u8>` for a single instruction (be it a word or data)
///
/// # Data Representation
///
/// In an effort to make PumpkinScript interpretation efficient,
/// we are not introducing enums or structures to represent instructions.
/// Instead, their byte representation is kept.
///
/// * `<len @ 0..120u8> [_;len]` — byte arrays of up to 120 bytes can have their size indicated
/// in the first byte, followed by that size's number of bytes
/// * `<121u8> <len u8> [_; len]` — byte array from 121 to 255 bytes can have their size indicated
/// in the second byte, followed by that size's number of bytes, with `121u8` as the first byte
/// * `<122u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size
/// indicated in the second and third bytes (u16), followed by that size's number of bytes,
/// with `122u8` as the first byte
/// * `<123u8> <len u32> [_; len]` — byte array from 65536 to 4294967296 bytes can have their
/// size indicated in the second, third, fourth and fifth bytes (u32), followed by that size's
/// number of bytes, with `123u8` as the first byte
/// * `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following
/// byte array of `len & 128u8` length (len without the highest bit set) is considered a word.
/// Length must be greater than zero.
///
/// The rest of tags (`124u8` to `128u8`) are reserved for future use.
///
pub type Instruction = Vec<u8>;
/// `Data` is used as another alias for `Vec<u8>` to distinguish the fact that this is only data
/// and has already been stripped of encoding. Useful for representing data on the stack.
pub type Data = Vec<u8>;
/// `Stack` is an alias for `Vec<Data>`
pub type Stack = Vec<Data>;
/// `Program` is a vector of instructions.
pub type Program = Vec<Instruction>;

/// `Error` represents an enumeration of possible `Executor` errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An attempt to get a value off the top of the stack was made,
    /// but the stack was empty.
    EmptyStack(Program),
    /// Word is unknown
    UnknownWord(Instruction, Stack, Program),
    /// Binary format decoding failed
    DecodingError(Instruction),
    /// Program parsing error
    ProgramParsingError(ParseError),
}

/// Executes a program
///
/// A curious observer might notice that this function returns
/// a future. The reason for that is that some of the words inherently take
/// an non-trivial amount of time to finish (I/O, waiting, etc.). In order
/// to avoid writing blocking code and limiting the capacity of the executor,
/// all executions are represented through futures. In trivial cases those futures
/// can be immediately resolved because they never really involved any async
/// operations.
pub fn execute(stack: Stack, program: Program) -> BoxFuture<(Stack, Program), Error> {
    run((stack, program))
}

macro_rules! pop_or_fail {
    ($stack:expr, $program:expr) => {
      match $stack.pop() {
        Some(v) => v,
        None => return future::err(Error::EmptyStack($program)).boxed()
      }
    };
}

macro_rules! push {
    ($stack:expr, $slice:expr) => {
    {
       let mut vec = Vec::new();
       vec.extend_from_slice($slice);
       $stack.push(vec);
    }
    };
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Incomplete,
    Err(u32),
    UnknownErr,
}


mod binparser {
    use nom::{IResult, Needed, ErrorKind};
    use script::{Program, ParseError};


    fn to_vec(s: &[u8]) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(s);
        vec
    }

    pub fn word_tag(i: &[u8]) -> IResult<&[u8], u8> {
        if i.len() < 1 {
            IResult::Incomplete(Needed::Size(1))
        } else if i[0] & 128 != 128 {
            IResult::Error(ErrorKind::Custom(128))
        } else {
            IResult::Done(&i[0..], i[0] - 128 + 1)
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
    named!(data<Vec<u8>>, do_parse!(
                                  data: length_bytes!(alt!(micro_length |
                                                           byte_length  |
                                                           small_length |
                                                           big_length))  >>
                                        (to_vec(data))));

    named!(word<Vec<u8>>, do_parse!(
                                 word: length_bytes!(word_tag)      >>
                                       (to_vec(word))));

    named!(split_code<Vec<Vec<u8>>>, many0!(alt!(word | data)));

    /// Parse single Vec<u8> into separate instructions (a program)
    pub fn parse(code: Vec<u8>) -> Result<Program, ParseError> {
        match split_code(code.as_slice()) {
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
        fn test() {
            let v = parse_text("0x10 DUP").unwrap();
            let mut vec = Vec::new();
            for mut item in v {
                vec.append(&mut item);
            }
            assert_eq!(parse(vec).unwrap(), parse_text("0x10 DUP").unwrap());
        }

        #[test]
        fn test_byte() {
            let mut byte_sized_sequence: String = "0x".to_owned();
            for _ in 1..200 {
                byte_sized_sequence.push_str("AA");
            }
            let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
            let mut vec = Vec::new();
            for mut item in v {
                vec.append(&mut item);
            }
            assert_eq!(parse(vec).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
        }

        #[test]
        fn test_small() {
            let mut byte_sized_sequence: String = "0x".to_owned();
            for _ in 1..300 {
                byte_sized_sequence.push_str("AA");
            }
            let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
            let mut vec = Vec::new();
            for mut item in v {
                vec.append(&mut item);
            }
            assert_eq!(parse(vec).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
        }

        #[test]
        fn test_big() {
            let mut byte_sized_sequence: String = "0x".to_owned();
            for _ in 1..70000 {
                byte_sized_sequence.push_str("AA");
            }
            let v = parse_text(byte_sized_sequence.as_ref()).unwrap();
            let mut vec = Vec::new();
            for mut item in v {
                vec.append(&mut item);
            }
            assert_eq!(parse(vec).unwrap(), parse_text(byte_sized_sequence.as_ref()).unwrap());
        }

    }

}

pub use self::binparser::parse as parse_bin;

fn run(tuple: (Stack, Program)) -> BoxFuture<(Stack, Program), Error> {
    let (mut stack, mut program) = tuple;
    if program.len() > 0 {
        let instruction = program.remove(0);
        match instruction.clone().as_slice() {
            // data
            &[sz @ 0u8...120u8, ref body..] if body.len() == sz as usize => push!(stack, body),
            &[121u8, sz, ref body..] if body.len() == sz as usize => push!(stack, body),
            &[122u8, sz0, sz1, ref body..] if body.len() ==
                                              (sz0 as usize) << 8 | (sz1 as usize) => {
                push!(stack, body)
            }
            &[123u8, sz0, sz1, sz2, sz3, ref body..] if body.len() ==
                                                        (sz0 as usize) << 24 |
                                                        (sz1 as usize) << 16 |
                                                        (sz2 as usize) << 8 |
                                                        (sz3 as usize) => push!(stack, body),
            // words
            &[ref body..] if body == DROP => {
                let _ = pop_or_fail!(stack, program);
            }
            &[ref body..] if body == DUP => {
                let v = pop_or_fail!(stack, program);
                let v1 = v.clone();
                stack.push(v);
                stack.push(v1);
            }
            &[ref body..] if body == SWAP => {
                let a = pop_or_fail!(stack, program);
                let b = pop_or_fail!(stack, program);
                stack.push(a);
                stack.push(b);
            }
            &[ref body..] if body == ROT => {
                let a = pop_or_fail!(stack, program);
                let b = pop_or_fail!(stack, program);
                let c = pop_or_fail!(stack, program);
                stack.push(b);
                stack.push(a);
                stack.push(c);
            }
            &[ref body..] if body == OVER => {
                let a = pop_or_fail!(stack, program);
                let b = pop_or_fail!(stack, program);
                let c = b.clone();
                stack.push(b);
                stack.push(a);
                stack.push(c);
            }
            &[ref body..] if body == CONCAT => {
                let mut a = pop_or_fail!(stack, program);
                let mut b = pop_or_fail!(stack, program);
                b.append(&mut a);
                stack.push(b);
            }
            &[ref body..] if body == EVAL => {
                let code = pop_or_fail!(stack, program);
                match parse_bin(code) {
                    Ok(p) => return execute(stack, p).and_then(|(s, _)| run((s, program))).boxed(),
                    Err(err) => return future::err(Error::ProgramParsingError(err)).boxed(),
                }
            }
            &[sz @ 129u8...255u8, ref body..] if body.len() == (sz ^ 128u8) as usize => {
                let mut vec = Vec::new();
                vec.extend_from_slice(body);
                program.insert(0, instruction);
                return future::err(Error::UnknownWord(vec, stack, program)).boxed();
            }
            // decoding error
            data => {
                let mut vec = Vec::new();
                vec.extend_from_slice(data);
                return future::err(Error::DecodingError(vec)).boxed();
            }
        }
    }
    if program.is_empty() {
        return future::ok((stack, program)).boxed();
    } else {
        return future::ok((stack, program)).and_then(run).boxed();
    }
}

#[cfg(test)]
mod tests {
    use script::{Error, parse, execute};

    use futures::Future;

    #[quickcheck]
    fn push_micro(size: u8) -> bool {
        if size > 120 {
            // micros don't handle anything about 120
            return true;
        }
        let mut vec: Vec<u8> = Vec::new();
        let mut data = Vec::with_capacity(size as usize);
        data.resize(size as usize, 0u8);
        vec.push(size);
        vec.extend_from_slice(data.as_slice());
        assert_eq!(vec.len(), size as usize + 1);
        let (mut stack, _) = execute(Vec::new(), vec![vec]).wait().unwrap();
        stack.pop().unwrap() == data.as_slice()
    }

    #[quickcheck]
    fn push_byte(size: u8) -> bool {
        let mut vec: Vec<u8> = Vec::new();
        let mut data = Vec::with_capacity(size as usize);
        data.resize(size as usize, 0u8);
        vec.push(121u8);
        vec.push(size);
        vec.extend_from_slice(data.as_slice());
        assert_eq!(vec.len(), size as usize + 2);
        let (mut stack, _) = execute(Vec::new(), vec![vec]).wait().unwrap();
        stack.pop().unwrap() == data.as_slice()
    }

    #[quickcheck]
    fn push_small(size: u16) -> bool {
        let mut vec: Vec<u8> = Vec::new();
        let mut data = Vec::with_capacity(size as usize);
        data.resize(size as usize, 0u8);
        vec.push(122u8);
        vec.push((size >> 8) as u8);
        vec.push(size as u8);
        vec.extend_from_slice(data.as_slice());
        assert_eq!(vec.len(), size as usize + 3);
        let (mut stack, _) = execute(Vec::new(), vec![vec]).wait().unwrap();
        stack.pop().unwrap() == data.as_slice()
    }

    #[quickcheck]
    fn push_big(size: u32) -> bool {
        let mut vec: Vec<u8> = Vec::new();
        let mut data = Vec::with_capacity(size as usize);
        data.resize(size as usize, 0u8);
        vec.push(123u8);
        vec.push((size >> 24) as u8);
        vec.push((size >> 16) as u8);
        vec.push((size >> 8) as u8);
        vec.push(size as u8);
        vec.extend_from_slice(data.as_slice());
        assert_eq!(vec.len(), size as usize + 5);
        let (mut stack, _) = execute(Vec::new(), vec![vec]).wait().unwrap();
        stack.pop().unwrap() == data.as_slice()
    }

    #[test]
    fn unknown_word() {
        let f = execute(Vec::new(), parse("XXX").unwrap()).wait();
        assert!(f.is_err());
        if let Error::UnknownWord(instruction, _, mut program) = f.err().unwrap() {
            assert_eq!(instruction, vec![b'X', b'X', b'X']);
            // the instruction is still on the list
            assert_eq!(program.pop().unwrap(), vec![0x83, b'X', b'X', b'X']);
        }
    }

    #[test]
    fn invalid_word_encoding() {
        let f = execute(Vec::new(), vec![vec![0x84, b'X', b'X', b'X']]).wait();
        assert!(f.is_err());
        assert!(matches!(f.err(), Some(Error::DecodingError(_))));
        let f = execute(Vec::new(), vec![vec![0x80, b'X', b'X', b'X']]).wait();
        assert!(f.is_err());
        assert!(matches!(f.err(), Some(Error::DecodingError(_))));
    }

    #[test]
    fn drop() {
        let (mut stack, _) = execute(Vec::new(), parse("0x010203 DROP").unwrap()).wait().unwrap();
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to drop should result in an error
        assert!(matches!(execute(stack, parse("DROP").unwrap()).wait().err(),
        Some(Error::EmptyStack(_))));
    }

    #[test]
    fn dup() {
        let (mut stack, _) = execute(Vec::new(), parse("0x010203 DUP").unwrap()).wait().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to duplicate should result in an error
        assert!(matches!(execute(stack, parse("DUP").unwrap()).wait().err(),
                         Some(Error::EmptyStack(_))));
    }

    #[test]
    fn swap() {
        let (mut stack, _) =
            execute(Vec::new(), parse("0x010203 0x030201 SWAP").unwrap()).wait().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop().unwrap(), vec![3, 2, 1]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to swap should result in an error
        assert!(matches!(execute(stack, parse("SWAP").unwrap()).wait().err(),
        Some(Error::EmptyStack(_))));
    }

    #[test]
    fn rot() {
        let (mut stack, _) =
            execute(Vec::new(), parse("0x010203 0x030201 0x00 ROT").unwrap()).wait().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop().unwrap(), vec![0]);
        assert_eq!(stack.pop().unwrap(), vec![3, 2, 1]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to rotate should result in an error
        assert!(matches!(execute(stack, parse("ROT").unwrap()).wait().err(),
                         Some(Error::EmptyStack(_))));
    }

    #[test]
    fn over() {
        let (mut stack, _) =
            execute(Vec::new(), parse("0x010203 0x00 OVER").unwrap()).wait().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop().unwrap(), vec![0]);
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to rotate should result in an error
        assert!(matches!(execute(stack, parse("OVER").unwrap()).wait().err(),
        Some(Error::EmptyStack(_))));
    }

    #[test]
    fn concat() {
        let (mut stack, _) =
            execute(Vec::new(), parse("0x10 0x20 CONCAT").unwrap()).wait().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![0x10, 0x20]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to rotate should result in an error
        assert!(matches!(execute(stack, parse("CONCAT").unwrap()).wait().err(),
        Some(Error::EmptyStack(_))));
    }

    #[test]
    fn eval() {
        let (mut stack, _) = execute(Vec::new(),
                                     parse("[0x01 DUP [DUP] EVAL] EVAL DROP").unwrap())
            .wait()
            .unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1]);
        assert_eq!(stack.pop().unwrap(), vec![1]);
        assert_eq!(stack.pop(), None);

        // now that the stack is empty, at attempt
        // to rotate should result in an error
        assert!(matches!(execute(stack, parse("EVAL").unwrap()).wait().err(),
        Some(Error::EmptyStack(_))));
    }

    #[test]
    fn invalid_eval() {
        let f = execute(Vec::new(), parse("0x10 EVAL").unwrap()).wait();
        assert!(f.is_err());
        assert!(matches!(f.err(), Some(Error::ProgramParsingError(_))));
    }

}


mod textparser {
    use nom::{IResult, ErrorKind};
    use script::{Program, ParseError};
    use nom::is_hex_digit;

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
                             prog: delimited!(char!('['), program, char!(']')) >>
                                   (program_to_vec(prog))));
    named!(item<Vec<u8>>, alt!(binary | string | code | word));
    named!(program<Vec<Vec<u8>>>, separated_list!(tag!(" "), item));

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
            let dup = [4, 0x83, b'D', b'U', b'P'];
            let mut vec: Vec<&[u8]> = Vec::new();
            vec.push(&dup);
            assert_eq!(script, vec);
        }

    }
}

pub use self::textparser::parse;
