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
//! # Features
//!
//! * Binary and text (human readable & writable) forms
//! * No types, just byte arrays
//! * Dynamic code evaluation
//! * Zero-copy interpretation (where feasible)
//!


use alloc::heap;


/// `word!` macro is used to define a built-in word, its signature (if applicable)
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

// How to write a new built-in word:
// 1. Add `word!(...)` to define a constant
// 2. Document it
// 3. Write a test in mod tests
// 4. Add `handle_word` function in VM and list it in `match_words!()` macro
//    invocation in VM::pass

// Category: Stack

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

// Category: Byte arrays

/// `CONCAT` takes two topmost items and concatenates them, and
/// pushes result to the top of the stack
word!(CONCAT, (a, b => c), b"\x86CONCAT");

// category: Control flow

/// `EVAL` takes the topmost item and evaluates it as a PumpkinScript
/// program on the current stack
word!(EVAL, b"\x84EVAL");

/// # Data Representation
///
/// In an effort to keep PumpkinScript dead simple, we are not introducing enums
/// or structures to represent instructions (although some argued that we rather should).
/// Instead, their binary form is kept.
///
/// Data push instructions:
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
///
/// Word:
///
/// * `<len @ 129u8..255u8> [_; len ^ 128u8]` — if `len` is greater than `128u8`, the following
/// byte array of `len & 128u8` length (len without the highest bit set) is considered a word.
/// Length must be greater than zero.
///
/// The rest of tags (`124u8` to `128u8`) are reserved for future use.
///
pub type Program = Vec<u8>;

/// `Error` represents an enumeration of possible `Executor` errors.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// An attempt to get a value off the top of the stack was made,
    /// but the stack was empty.
    EmptyStack,
    /// Word is unknown
    UnknownWord,
    /// Binary format decoding failed
    DecodingError,
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

/// Initial stack size
pub const STACK_SIZE: usize = 32_768;
/// Initial heap size
pub const HEAP_SIZE: usize = 32_768;

/// Env is a representation of a stack and the heap.
///
/// Doesn't need to be used directly as it's primarily
/// used by [`VM`](struct.VM.html)
pub struct Env<'a> {
    stack: Vec<&'a [u8]>,
    stack_size: usize,
    heap: *mut u8,
    heap_size: usize,
    heap_align: usize,
    heap_ptr: usize,
}

const _EMPTY: &'static [u8] = b"";

use std::slice;
use std::mem;

impl<'a> Env<'a> {
    /// Creates an environment with [an empty stack of default size](constant.STACK_SIZE.html)
    pub fn new() -> Self {
        Env::new_with_stack_size(STACK_SIZE)
    }

    /// Creates an environment with an empty stack of specific size
    pub fn new_with_stack_size(size: usize) -> Self {
        Env::new_with_stack(vec![_EMPTY; size], 0)
    }

    /// Creates an environment with an existing stack and a pointer to the
    /// topmost element (stack_size)
    ///
    /// This function is useful for working with result stacks received from
    /// [VM](struct.VM.html)
    pub fn new_with_stack(stack: Vec<&'a [u8]>, stack_size: usize) -> Self {
        Env {
            stack: stack,
            stack_size: stack_size,
            heap: unsafe { heap::allocate(HEAP_SIZE, mem::align_of::<u8>()) },
            heap_size: HEAP_SIZE,
            heap_align: mem::align_of::<u8>(),
            heap_ptr: 0,
        }
    }

    /// Returns the entire stack
    #[inline]
    pub fn stack(&self) -> &[&'a [u8]] {
        &self.stack.as_slice()[0..self.stack_size as usize]
    }

    /// Returns top of the stack without removing it
    #[inline]
    pub fn stack_top(&self) -> Option<&'a [u8]> {
        if self.stack_size == 0 {
            None
        } else {
            Some(self.stack.as_slice()[self.stack_size as usize - 1])
        }
    }

    /// Removes the top of the stack and returns it
    #[inline]
    pub fn pop(&mut self) -> Option<&'a [u8]> {
        if self.stack_size == 0 {
            None
        } else {
            let val = Some(self.stack.as_slice()[self.stack_size as usize - 1]);
            self.stack.as_mut_slice()[self.stack_size as usize - 1] = _EMPTY;
            self.stack_size -= 1;
            val
        }
    }

    /// Pushes value on top of the stack
    #[inline]
    pub fn push(&mut self, data: &'a [u8]) {
        self.stack.as_mut_slice()[self.stack_size as usize] = data;
        self.stack_size += 1;
        if self.stack_size == self.stack.len() {
            self.stack.reserve(STACK_SIZE);
        }
    }

    /// Allocates a slice off the Env-specific heap. Will be collected
    /// once this Env is dropped.
    pub fn alloc(&mut self, len: usize) -> &'a mut [u8] {
        let mut space = unsafe { slice::from_raw_parts_mut(self.heap, self.heap_size) };
        if self.heap_ptr + len > self.heap_size {
            unsafe {
                heap::reallocate(self.heap,
                                 self.heap_size,
                                 self.heap_size + HEAP_SIZE,
                                 self.heap_align);
            }
        }
        let slice = &mut space[self.heap_ptr..self.heap_ptr + len];
        self.heap_ptr += len;
        slice
    }
}

impl<'a> Drop for Env<'a> {
    fn drop(&mut self) {
        unsafe {
            heap::deallocate(self.heap, self.heap_size, self.heap_align);
        }
    }
}

use nom;

#[inline]
fn offset_by_size(size: usize) -> usize {
    match size {
        0...120 => 1,
        120...255 => 2,
        255...65535 => 3,
        65536...4294967296 => 5,
        _ => unreachable!(),
    }
}

macro_rules! write_size_into_slice {
    ($size:expr, $slice: expr) => {
     match $size {
        0...120 => {
            $slice[0] = $size as u8;
            1
        }
        121...255 => {
            $slice[0] = 121u8;
            $slice[1] = $size as u8;
            2
        }
        256...65535 => {
            $slice[0] = 122u8;
            $slice[1] = ($size >> 8) as u8;
            $slice[2] = $size as u8;
            3
        }
        65536...4294967296 => {
            $slice[0] = 123u8;
            $slice[1] = ($size >> 24) as u8;
            $slice[2] = ($size >> 16) as u8;
            $slice[3] = ($size >> 8) as u8;
            $slice[4] = $size as u8;
            5
        }
        _ => unreachable!(),
    }
    };
}

macro_rules! data {
    ($ptr:expr) => {
        {
          let (_, size) = binparser::data_size($ptr).unwrap();
          (&$ptr[offset_by_size(size)..$ptr.len()], size)
        }
    };
}

macro_rules! handle_words {
    ($env: expr, $word: expr, $res: ident, [ $($name: ident),* ], $block: expr) => {
    {
      $(
        match VM::$name($env, $word) {
          Err(Error::UnknownWord) => (),
          Err(err) => return Err(err),
          Ok($res) => $block
        }
      )*
      return Err(Error::UnknownWord)
    }
    };
}

use std::sync::mpsc;
use snowflake::ProcessUniqueId;
use std;

pub type EnvId = ProcessUniqueId;

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;

/// Communication messages used to talk with the [VM](struct.VM.html) thread.
#[derive(Clone, Debug)]
pub enum RequestMessage<'a> {
    /// Requests scheduling a new environment with a given
    /// id and a program.
    ScheduleEnv(EnvId, Vec<u8>, Sender<ResponseMessage<'a>>),
    /// An internal message that schedules an execution of
    /// the next instruction in an identified environment on
    /// the next 'tick'
    RescheduleEnv(EnvId),
    /// Requests VM shutdown
    Shutdown,
}

/// Messages received from the [VM](struct.VM.html) thread.
#[derive(Clone, Debug)]
pub enum ResponseMessage<'a> {
    /// Notifies of successful environment termination with
    /// an id, stack and top of the stack pointer.
    EnvTerminated(EnvId, Vec<&'a [u8]>, usize),
    /// Notifies of abnormal environment termination with
    /// an id, error, stack and top of the stack pointer.
    EnvFailed(EnvId, Error, Vec<&'a [u8]>, usize),
}

pub type TrySendError<T> = std::sync::mpsc::TrySendError<T>;

use std::collections::HashMap;


/// VM is a PumpkinScript scheduler and interpreter. This is the
/// most central part of this module.
///
/// # Example
///
/// ```no_run
/// let mut vm = VM::new();
///
/// let sender = vm.sender();
/// let handle = thread::spawn(move || {
///     vm.run();
/// });
/// let script = parse($script).unwrap();
/// let (callback, receiver) = mpsc::channel::<ResponseMessage>();
/// let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), script.clone(), callback));
/// match receiver.recv() {
///     Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
///         let _ = sender.send(RequestMessage::Shutdown);
///         // success
///         // ...
///     }
///     Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
///         let _ = sender.send(RequestMessage::Shutdown);
///         // failure
///         // ...
///     }
///     Err(err) => {
///         panic!("recv error: {:?}", err);
///     }
/// }
/// ```
pub struct VM<'a> {
    inbox: Receiver<RequestMessage<'a>>,
    sender: Sender<RequestMessage<'a>>,
    loopback: Sender<RequestMessage<'a>>,
    envs: HashMap<EnvId, (Env<'a>, Vec<u8>, Sender<ResponseMessage<'a>>)>,
}

unsafe impl<'a> Send for VM<'a> {}

impl<'a> VM<'a> {
    /// Creates an instance of VM with three communication channels:
    ///
    /// * Response sender
    /// * Internal sender
    /// * Request receiver
    pub fn new()
               -> Self {
        let (sender, receiver) = mpsc::channel::<RequestMessage<'a>>();
        VM {
            inbox: receiver,
            sender: sender.clone(),
            loopback: sender.clone(),
            envs: HashMap::new(),
        }
    }

    pub fn sender(&self) -> Sender<RequestMessage<'a>> {
        self.sender.clone()
    }

    /// Scheduler thread. It is supposed to be running in a separate thread
    ///
    /// The scheduler handles all incoming and internal messages. Once at least one
    /// program is scheduled (`ScheduleEnv`), it will create an [Env](struct.Env.html) for
    /// it and reschedule for execution (`RescheduleEnv`), at which time it will execute
    /// one instruction. This way it can execute multiple scripts at the same time.
    ///
    /// Once an environment execution has been terminated, a message will be sent,
    /// depending on the result (`EnvTerminated` or `EnvFailed`)
    pub fn run(&mut self) {
        loop {
            match self.inbox.recv() {
                Err(err) => panic!("error receiving: {:?}", err),
                Ok(RequestMessage::Shutdown) => break,
                Ok(RequestMessage::ScheduleEnv(pid, program, chan)) => {
                    let env = Env::new();
                    self.envs.insert(pid, (env, program, chan));
                    let _ = self.loopback.send(RequestMessage::RescheduleEnv(pid));
                }
                Ok(RequestMessage::RescheduleEnv(pid)) => {
                    if let Some((mut env, mut program, chan)) = self.envs.remove(&pid) {
                        match self.pass(&mut env, &mut program) {
                            Err(err) => {
                                let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                 err,
                                                                 Vec::from(env.stack()),
                                                                 env.stack_size));
                            }
                            Ok(Some(program)) => {
                                self.envs.insert(pid, (env, program, chan));
                                let _ = self.loopback.send(RequestMessage::RescheduleEnv(pid));
                            }
                            Ok(None) => {
                                let _ = chan.send(ResponseMessage::EnvTerminated(pid,
                                                                     Vec::from(env.stack()),
                                                                     env.stack_size));
                            }
                        };
                    }
                }
            }
        }
    }

    fn pass(&mut self, mut env: &mut Env, program: &mut Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let mut slice = env.alloc(program.len());
        for i in 0..program.len() {
            slice[i] = program[i];
        }
        if let nom::IResult::Done(_, data) = binparser::data(slice) {
            env.push(data);
            let rest = program.split_off(data.len());
            return Ok(match rest.len() {
                0 => None,
                _ => Some(rest),
            });
        } else if let nom::IResult::Done(_, word) = binparser::word(slice) {
            handle_words!(env, word, res,
                                     [handle_drop, handle_dup, handle_swap,
                                      handle_rot, handle_over, handle_eval,
                                      handle_concat], {
                let rest = match res {
                        Some(code_injection) => {
                            let mut vec = Vec::from(code_injection);
                            let mut rest_0 = program.split_off(word.len());
                            vec.append(&mut rest_0);
                            vec
                        }
                        None => program.split_off(word.len())
                    };
                return Ok(match rest.len() {
                    0 => None,
                    _ => Some(rest)
                });
            });
        } else {
            return Err(Error::DecodingError);
        }
    }

    #[inline]
    fn handle_dup(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == DUP {
            match env.pop() {
                None => return Err(Error::EmptyStack),
                Some(v) => {
                    env.push(v);
                    env.push(v);
                    Ok(None)
                }
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_swap(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == SWAP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err(Error::EmptyStack);
            }

            env.push(a.unwrap());
            env.push(b.unwrap());

            Ok(None)
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_over(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == OVER {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err(Error::EmptyStack);
            }

            env.push(b.unwrap());
            env.push(a.unwrap());
            env.push(b.unwrap());

            Ok(None)
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_rot(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == ROT {
            let a = env.pop();
            let b = env.pop();
            let c = env.pop();

            if a.is_none() || b.is_none() || c.is_none() {
                return Err(Error::EmptyStack);
            }

            env.push(b.unwrap());
            env.push(a.unwrap());
            env.push(c.unwrap());

            Ok(None)
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_drop(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == DROP {
            match env.pop() {
                None => return Err(Error::EmptyStack),
                _ => Ok(None),
            }
        } else {
            Err(Error::UnknownWord)
        }
    }


    #[inline]
    fn handle_concat(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == CONCAT {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err(Error::EmptyStack);
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            let (a1_, size_a) = data!(a1);
            let (b1_, size_b) = data!(b1);

            let size = a1_.len() + b1_.len();

            let mut slice = env.alloc(size + offset_by_size(size_a + size_b));
            let mut offset = write_size_into_slice!(size, slice);

            for byte in b1_ {
                slice[offset] = *byte;
                offset += 1
            }

            for byte in a1_ {
                slice[offset] = *byte;
                offset += 1
            }

            env.push(slice);

            Ok(None)
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_eval(env: &mut Env, word: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        if word == EVAL {
            match env.pop() {
                None => return Err(Error::EmptyStack),
                Some(v) => {
                    let (code, _) = data!(v);
                    Ok(Some(Vec::from(code)))
                }
            }
        } else {
            Err(Error::UnknownWord)
        }
    }
}



#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use script::{Env, VM, Error, RequestMessage, ResponseMessage, EnvId, parse};
    use std::thread;
    use std::sync::mpsc;

    macro_rules! eval {
        ($script: expr, $env: ident, $expr: expr) => {
           eval!($script, $env, _result, $expr);
        };
        ($script: expr, $env: ident, $result: pat, $expr: expr) => {
          {
            let mut vm = VM::new();

            let sender = vm.sender();
            let handle = thread::spawn(move || {
                vm.run();
            });
            let script = parse($script).unwrap();
            let (callback, receiver) = mpsc::channel::<ResponseMessage>();
            let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), script.clone(), callback));
            match receiver.recv() {
               Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                  let _ = sender.send(RequestMessage::Shutdown);
                  let $result = Ok::<(), Error>(());
                  let mut $env = Env::new_with_stack(stack, stack_size);
                  $expr;
               }
               Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                  let _ = sender.send(RequestMessage::Shutdown);
                  let $result = Err::<(), Error>(err);
                  let mut $env = Env::new_with_stack(stack, stack_size);
                  $expr;
               }
               Err(err) => {
                  panic!("recv error: {:?}", err);
               }
            }
            let _ = handle.join();
          }
        };
    }

    #[test]
    fn drop() {
        eval!("0x010203 DROP", env, {
            assert_eq!(env.pop(), None);
        });

        eval!("DROP", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }

    #[test]
    fn dup() {
        eval!("0x010203 DUP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
            assert_eq!(env.pop(), None);
        });

        eval!("DUP", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });
    }

    #[test]
    fn swap() {
        eval!("0x010203 0x030201 SWAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x030201").unwrap());
            assert_eq!(env.pop(), None);
        });

        eval!("SWAP", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("0x10 SWAP", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }


    #[test]
    fn rot() {
        eval!("0x010203 0x030201 0x00 ROT", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x00").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x030201").unwrap());
            assert_eq!(env.pop(), None);
        });

        eval!("0x010203 0x030201 ROT", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("0x010203 ROT", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("ROT", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }

    #[test]
    fn over() {
        eval!("0x010203 0x00 OVER", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x00").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x010203").unwrap());
        });

        eval!("0x00 OVER", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("OVER", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }

    #[test]
    fn concat() {
        eval!("0x10 0x20 CONCAT", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x1020").unwrap());
            assert_eq!(env.pop(), None);
        });

        eval!("0x20 CONCAT", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("CONCAT", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });
    }

    #[test]
    fn eval() {
        eval!("[0x01 DUP [DUP] EVAL] EVAL DROP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x01").unwrap());
            assert_eq!(Vec::from(env.pop().unwrap()), parse("0x01").unwrap());
            assert_eq!(env.pop(), None);
        });

        eval!("EVAL", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });
    }

    #[test]
    fn invalid_eval() {
        eval!("0x10 EVAL", env, result, {
            assert!(result.is_err());
            assert!(matches!(result.err(), Some(Error::DecodingError)));
        });
    }

}
