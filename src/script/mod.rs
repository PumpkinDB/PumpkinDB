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

/// Parse-related error
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Incomplete input
    Incomplete,
    /// Error with a code
    Err(u32),
    /// Unknown error
    UnknownErr,
}

mod binparser;
pub use self::binparser::parse as parse_bin;

mod textparser;
pub use self::textparser::parse;


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