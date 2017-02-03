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
//! for indexing — in a compact, unambiguous and composable form? Or even for recording data itself?
//! Well, that's where the idea to use something like a Forth-like script was born.
//!
//! Instead of devising custom protocols for talking to PumpkinDB, the protocol of communication has
//! become a pipeline to a script executor.
//!
//! So, for example, a command/events set can be recorded with something like this (not an actual script,
//! below is pseudocode):
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
//! The main entry point for executing PumpkinScript is [`Env`](struct.Env.html) via
//! the [`Executor`](trait.Executor.html) trait.
//!
//! # Examples
//!
//! ```
//! let x = [65, 66];
//! let mut env = Env::new();
//! env.push(&x);
//! env.execute(DROP).wait().unwrap();
//! assert_eq!(env.pop(), None);
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


/// `Instruction` is a type alias for a single instruction (be it a word or data)
///
/// In an effort to make PumpkinScript interpretation efficient,
/// we are not introducing enums or structures to represent instructions.
/// Instead, their byte representation is kept.
///
/// # Representation
///
/// `<len @ 0..120u8> [_;len]` — byte arrays of up to 120 bytes can have their size indicated in the first byte,
/// followed by that size's number of bytes
/// `<121u8> <len u8> [_; len]` — byte array from 121 to 255 bytes can have their size indicated in the second byte,
/// followed by that size's number of bytes, with `121u8` as the first byte
/// `<122u8> <len u16> [_; len]` — byte array from 256 to 65535 bytes can have their size indicated in the second and third bytes (u16),
/// followed by that size's number of bytes, with `122u8` as the first byte
/// `<123u8> <len u32> [_; len]` — byte array from 65536 to 4294967296 bytes can have their size indicated in the second, third,
/// fourth and fifth bytes (u32), followed by that size's number of bytes, with `123u8` as the first byte
/// `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following byte array of `len & 128u8` length
/// (len without the highest bit set) is considered a word. Length must be greater than zero.
///
/// The rest of tags (`124u8` to `128u8`) are reserved for future use.
///
type Instruction = [u8];
/// Data is used as another alias to a byte slice to distinguish the fact that this is only data
/// and has already been stripped of encoding. Useful for representing data on the stack.
type Data = [u8];

/// `Error` represents an enumeration of possible `Executor` errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An attempt to get a value off the top of the stack was made,
    /// but the stack was empty.
    EmptyStack,
    /// Word is unknown
    UnknownWord,
    /// Binary format decoding failed
    DecodingError,
}

/// `Executor` is a trait that serves as an interface for executing scripts
pub trait Executor<'a> {
    /// Executes one instruction (word or push)
    fn execute(&mut self, instruction: &'a Instruction) -> BoxFuture<(), Error>;
}

/// `HasStack` is a trait that serves as an interface for accessing PumpkinScript
/// stack
trait HasStack<'a> {
    fn push(&mut self, &'a Data);
    fn pop(&mut self) -> Option<&'a Data>;
}

/// PumpkinScript environment. This is the structure typically used to run
/// the scripts.
pub struct Env<'a> {
    stack: Vec<&'a Data>,
}

impl<'a> Env<'a> {
    /// Creates a new PumpkinScript environment.
    fn new() -> Self {
        Env { stack: Vec::new() }
    }
}

macro_rules! pop_or_fail {
    ($stack:expr) => { match $stack.pop() { Some(v) => v, None => return future::err(Error::EmptyStack).boxed() } };
}

impl<'a> Executor<'a> for Env<'a> {
    fn execute(&mut self, instruction: &'a Instruction) -> BoxFuture<(), Error> {
        match instruction {
            // data
            &[sz @ 0u8...120u8, ref body..] if body.len() == sz as usize => self.push(body),
            &[121u8, sz, ref body..] if body.len() == sz as usize => self.push(body),
            &[122u8, sz0, sz1, ref body..] if body.len() == (sz0 as usize) << 8 | (sz1 as usize) => self.push(body),
            &[123u8, sz0, sz1, sz2, sz3, ref body..] if body.len() == (sz0 as usize) << 24 | (sz1 as usize) << 16 | (sz2 as usize) << 8 | (sz3 as usize) => self.push(body),
            // words
            &[ref body..] if body == DROP => {
                let _ = pop_or_fail!(self);
            }
            &[ref body..] if body == DUP => {
                let v = pop_or_fail!(self);
                self.push(v);
                self.push(v);
            },
            &[ref body..] if body == SWAP => {
                let a = pop_or_fail!(self);
                let b = pop_or_fail!(self);
                self.push(a);
                self.push(b);
            }
            &[ref body..] if body == ROT => {
                let a = pop_or_fail!(self);
                let b = pop_or_fail!(self);
                let c = pop_or_fail!(self);
                self.push(b);
                self.push(a);
                self.push(c);
            }
            // unknown word
            &[sz @ 129u8...255u8, ref body..] if body.len() == (sz ^ 128u8) as usize => return future::err(Error::UnknownWord).boxed(),
            // decoding error
            _ => return future::err(Error::DecodingError).boxed()
        }
        return future::ok(()).boxed();

    }
}

impl<'a> HasStack<'a> for Env<'a> {
    fn push(&mut self, v: &'a Data) {
        self.stack.push(v)
    }

    fn pop(&mut self) -> Option<&'a Data> {
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use script::{Env, HasStack, Executor, Error};
    use script::{DUP, DROP, SWAP, ROT};
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
        let slice = vec.as_slice();
        let mut env = Env::new();
        env.execute(slice).wait().unwrap();
        env.pop().unwrap() == data.as_slice()
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
        let slice = vec.as_slice();
        let mut env = Env::new();
        env.execute(slice).wait().unwrap();
        env.pop().unwrap() == data.as_slice()
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
        let slice = vec.as_slice();
        let mut env = Env::new();
        env.execute(slice).wait().unwrap();
        env.pop().unwrap() == data.as_slice()
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
        let slice = vec.as_slice();
        let mut env = Env::new();
        env.execute(slice).wait().unwrap();
        env.pop().unwrap() == data.as_slice()
    }

    #[test]
    fn unknown_word() {
        let mut env = Env::new();
        let f = env.execute(b"\x83XXX").wait();
        assert!(f.is_err());
        assert_eq!(f.err(), Some(Error::UnknownWord));
    }

    #[test]
    fn invalid_word_encoding() {
        let mut env = Env::new();
        let f = env.execute(b"\x84XXX").wait();
        assert!(f.is_err());
        assert_eq!(f.err(), Some(Error::DecodingError));
        let f = env.execute(b"\x80XXX").wait();
        assert!(f.is_err());
        assert_eq!(f.err(), Some(Error::DecodingError));
    }

    #[test]
    fn drop() {
        let x = [1, 2, 3];
        let mut env = Env::new();
        env.push(&x);
        env.execute(DROP).wait().unwrap();
        assert_eq!(env.pop(), None);

        // now that the stack is empty, at attempt
        // to drop should result in an error
        assert_eq!(env.execute(DROP).wait().err().unwrap(), Error::EmptyStack);
    }

    #[test]
    fn dup() {
        let x = [1, 2, 3];
        let mut env = Env::new();
        env.push(&x);
        env.execute(DUP).wait().unwrap();
        assert_eq!(env.pop().unwrap(), x);
        assert_eq!(env.pop().unwrap(), x);
        assert_eq!(env.pop(), None);

        // now that the stack is empty, at attempt
        // to duplicate should result in an error
        assert_eq!(env.execute(DUP).wait().err().unwrap(), Error::EmptyStack);
    }

    #[test]
    fn swap() {
        let x1 = [1, 2, 3];
        let x2 = [3, 2, 1];
        let mut env = Env::new();
        env.push(&x1);
        env.push(&x2);
        env.execute(SWAP).wait().unwrap();
        assert_eq!(env.pop().unwrap(), x1);
        assert_eq!(env.pop().unwrap(), x2);
        assert_eq!(env.pop(), None);

        // now that the stack is empty, at attempt
        // to swap should result in an error
        assert_eq!(env.execute(SWAP).wait().err().unwrap(), Error::EmptyStack);
    }

    #[test]
    fn rot() {
        let x1 = [1, 2, 3];
        let x2 = [3, 2, 1];
        let x3 = [0];
        let mut env = Env::new();
        env.push(&x1);
        env.push(&x2);
        env.push(&x3);
        env.execute(ROT).wait().unwrap();
        assert_eq!(env.pop().unwrap(), x1);
        assert_eq!(env.pop().unwrap(), x3);
        assert_eq!(env.pop().unwrap(), x2);
        assert_eq!(env.pop(), None);

        // now that the stack is empty, at attempt
        // to rotate should result in an error
        assert_eq!(env.execute(ROT).wait().err().unwrap(), Error::EmptyStack);
    }
}
