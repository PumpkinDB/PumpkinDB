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
//! * Zero-copy interpretation (where feasible; currently does not apply to the most
//!   important part, the storage itself as transactional model of LMDB precludes us
//!   from carrying these references outside of the scope of the transaction)
//!


use alloc::heap;

use num_bigint::BigUint;
use num_traits::{Zero, One};
use core::ops::Sub;
use std::cmp;

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
word!(DROP, (a => ), b"\x84DROP");
word!(DUP, (a => a, a), b"\x83DUP");
word!(SWAP, (a, b => b, a), b"\x84SWAP");
word!(ROT, (a, b, c  => b, c, a), b"\x83ROT");
word!(OVER, (a, b => a, b, a), b"\x84OVER");
word!(DEPTH, b"\x85DEPTH");

// Category: Byte arrays
word!(EQUALP, (a, b => c), b"\x86EQUAL?");
word!(LTP, (a, b => c), b"\x83LT?");
word!(GTP, (a, b => c), b"\x83GT?");
word!(CONCAT, (a, b => c), b"\x86CONCAT");

// Category: Control flow
word!(TIMES, b"\x85TIMES");
word!(EVAL, b"\x84EVAL");
word!(SET, b"\x83SET");
word!(SET_IMM, b"\x84SET!"); // internal word

// To add words that don't belong to a core set,
// add a module with a handler, and reference it in the VM's pass

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
/// `128u8` is reserved as a prefix to be followed by an internal VM's word (not to be accessible
/// to the end users).
///
/// The rest of tags (`124u8` to `127u8`) are reserved for future use.
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
    /// Database Error
    DatabaseError(lmdb::Error),
    /// Duplicate key
    DuplicateKey,
    /// Key not found
    UnknownKey,
    /// No active transaction
    NoTransaction,
    /// The item expected to be of a certain form,
    /// size, or other condition
    InvalidValue,
    /// An internal scheduler's error to indicate that currently
    /// executed environment should be rescheduled from the same point
    Reschedule,
    // Unable to (re)allocate the heap so the returning slice points to
    // unallocated memory.
    HeapAllocFailed,
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
/// Allocation-related outcomes
/// #[derive(Debug, PartialEq)]
enum AllocResult {
    Ok,
    Error
}
pub mod binparser;
pub use self::binparser::parse as parse_bin;

mod textparser;
pub use self::textparser::parse;

/// Initial stack size
pub const STACK_SIZE: usize = 32_768;
/// Initial heap size
pub const HEAP_SIZE: usize = 32_768;

use std::collections::BTreeMap;

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
    dictionary: BTreeMap<&'a [u8], &'a [u8]>
}

impl<'a> std::fmt::Debug for Env<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("Env()")
    }
}

unsafe impl<'a> Send for Env<'a> {}

const _EMPTY: &'static [u8] = b"";

use std::slice;
use std::mem;

impl<'a> Env<'a> {
    /// Creates an environment with [an empty stack of default size](constant.STACK_SIZE.html)
    pub fn new() -> Result<Self, Error> {
        Env::new_with_stack_size(STACK_SIZE)
    }

    /// Creates an environment with an empty stack of specific size
    pub fn new_with_stack_size(size: usize) -> Result<Self, Error> {
        Env::new_with_stack(vec![_EMPTY; size], 0)
    }

    /// Creates an environment with an existing stack and a pointer to the
    /// topmost element (stack_size)
    ///
    /// This function is useful for working with result stacks received from
    /// [VM](struct.VM.html)
    pub fn new_with_stack(stack: Vec<&'a [u8]>, stack_size: usize) -> Result<Self, Error> {
        unsafe {
            heap::allocate(HEAP_SIZE, mem::align_of::<u8>()).as_mut()
        }.and_then(|heap| {
                Some(Env {
                    stack: stack,
                    stack_size: stack_size,
                    heap: heap,
                    heap_size: HEAP_SIZE,
                    heap_align: mem::align_of::<u8>(),
                    heap_ptr: 0,
                    dictionary: BTreeMap::new()
                })
        }).ok_or(Error::HeapAllocFailed)
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
        // check if we are at capacity
        if self.stack_size == self.stack.len() {
            let mut vec = vec![_EMPTY; STACK_SIZE];
            self.stack.append(&mut vec);
        }
        self.stack.as_mut_slice()[self.stack_size] = data;
        self.stack_size += 1;
    }

    /// Allocates a slice off the Env-specific heap. Will be collected
    /// once this Env is dropped.
    pub fn alloc(&mut self, len: usize) -> Result<&'a mut [u8], Error> {
        if self.heap_ptr + len >= self.heap_size {
            let increase = cmp::max(len, HEAP_SIZE);
            unsafe {
                heap::reallocate(self.heap,
                                 self.heap_size,
                                 self.heap_size + increase,
                                 self.heap_align).as_mut()
            }.and_then(|heap| {
                self.heap = heap;
                Some(AllocResult::Ok)
            }).ok_or(AllocResult::Error)
        } else {
            Ok(AllocResult::Ok)
        }.and_then(|_| {
            let mut space = unsafe { slice::from_raw_parts_mut(self.heap, self.heap_size) };
            let slice = &mut space[self.heap_ptr..self.heap_ptr + len];
            self.heap_ptr += len;
            Ok(slice)
        }).or(Err(Error::HeapAllocFailed))
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
pub fn offset_by_size(size: usize) -> usize {
    match size {
        0...120 => 1,
        120...255 => 2,
        255...65535 => 3,
        65536...4294967296 => 5,
        _ => unreachable!(),
    }
}

include!("macros.rs");

use std::sync::mpsc;
use snowflake::ProcessUniqueId;
use std;

pub type EnvId = ProcessUniqueId;

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;

/// Communication messages used to talk with the [VM](struct.VM.html) thread.
#[derive(Debug)]
pub enum RequestMessage<'a> {
    /// Requests scheduling a new environment with a given
    /// id and a program.
    ScheduleEnv(EnvId, Vec<u8>, Sender<ResponseMessage<'a>>),
    /// An internal message that schedules an execution of
    /// the next instruction in an identified environment on
    /// the next 'tick'
    RescheduleEnv(EnvId, Vec<u8>, Env<'a>, Sender<ResponseMessage<'a>>),
    /// Requests VM shutdown
    Shutdown,
}

/// Messages received from the [VM](struct.VM.html) thread.
#[derive(Debug)]
pub enum ResponseMessage<'a> {
    /// Notifies of successful environment termination with
    /// an id, stack and top of the stack pointer.
    EnvTerminated(EnvId, Vec<&'a [u8]>, usize),
    /// Notifies of abnormal environment termination with
    /// an id, error, stack and top of the stack pointer.
    EnvFailed(EnvId, Error, Option<Vec<&'a [u8]>>, Option<usize>),
}

pub type TrySendError<T> = std::sync::mpsc::TrySendError<T>;

use lmdb;

pub mod storage;
pub mod timestamp_hlc;

/// VM is a PumpkinScript scheduler and interpreter. This is the
/// most central part of this module.
///
/// # Example
///
/// ```no_run
/// let mut vm = VM::new(&env, &db); // lmdb comes from outside
///
/// let sender = vm.sender();
/// let handle = thread::spawn(move || {
///     vm.run();
/// });
/// let script = parse("..script..");
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
    storage: storage::Handler<'a>,
    hlc: timestamp_hlc::Handler<'a>,
}

unsafe impl<'a> Send for VM<'a> {}

type PassResult<'a> = Result<(Env<'a>, Option<Vec<u8>>), (Env<'a>, Error)>;

const STACK_TRUE: &'static [u8] = b"\x01";
const STACK_FALSE: &'static [u8] = b"\x00";

impl<'a> VM<'a> {
    /// Creates an instance of VM with three communication channels:
    ///
    /// * Response sender
    /// * Internal sender
    /// * Request receiver
    pub fn new(db_env: &'a lmdb::Environment, db: &'a lmdb::Database<'a>) -> Self {
        let (sender, receiver) = mpsc::channel::<RequestMessage<'a>>();
        VM {
            inbox: receiver,
            sender: sender.clone(),
            loopback: sender.clone(),
            storage: storage::Handler::new(db_env, db),
            hlc: timestamp_hlc::Handler::new(),
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
                    match Env::new() {
                        Ok(env) => {
                            let _ = self.loopback
                                .send(RequestMessage::RescheduleEnv(pid, program, env, chan));
                        }
                        Err(err) => {
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         None,
                                                                         None));
                        }
                    }
                }
                Ok(RequestMessage::RescheduleEnv(pid, mut program, env, chan)) => {
                    match self.pass(env, &mut program, pid.clone()) {
                        Err((env, err)) => {
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         Some(Vec::from(env.stack())),
                                                                         Some(env.stack_size)));
                        }
                        Ok((env, Some(program))) => {
                            let _ = self.loopback
                                .send(RequestMessage::RescheduleEnv(pid, program, env, chan));
                        }
                        Ok((env, None)) => {
                            let _ = chan.send(ResponseMessage::EnvTerminated(pid,
                                                                     Vec::from(env.stack()),
                                                                     env.stack_size));
                        }
                    };
                }
            }
        }
    }

    fn pass(&mut self, mut env: Env<'a>, program: &mut Vec<u8>, pid: EnvId) -> PassResult<'a> {
        let mut slice = env.alloc(program.len());
        for i in 0..program.len() {
            slice[i] = program[i];
        }
        if let nom::IResult::Done(_, data) = binparser::data(slice) {
            env.push(&data[offset_by_size(data.len())..]);
            let rest = program.split_off(data.len());
            return Ok(match rest.len() {
                0 => (env, None),
                _ => (env, Some(rest)),
            });
        } else if let nom::IResult::Done(_, word) = binparser::word_or_internal_word(slice) {
            handle_words!(env,
                          program,
                          word,
                          res,
                          pid,
                          {self => handle_drop,
                           self => handle_dup,
                           self => handle_swap,
                           self => handle_rot,
                           self => handle_over,
                           self => handle_depth,
                           self => handle_ltp,
                           self => handle_gtp,
                           self => handle_equal,
                           self => handle_concat,
                           self => handle_times,
                           self => handle_eval,
                           self => handle_set,
                           // storage
                           self.storage => handle_write,
                           self.storage => handle_read,
                           self.storage => handle_assoc,
                           self.storage => handle_assocq,
                           self.storage => handle_retr,
                           self.storage => handle_commit,
                           // timestamping
                           self.hlc => handle_hlc,
                           self.hlc => handle_hlc_lc,
                           self.hlc => handle_hlc_tick,
                           self.hlc => handle_hlc_ltp,
                           self.hlc => handle_hlc_gtp
                           },
                          {
                              let (env_, rest) = match res {
                                  (env_, Some(code_injection)) => {
                                      let mut vec = Vec::from(code_injection);
                                      let mut rest_0 = program.split_off(word.len());
                                      vec.append(&mut rest_0);
                                      (env_, vec)
                                  }
                                  (env_, None) => (env_, program.split_off(word.len())),
                              };
                              return Ok(match rest.len() {
                                  0 => (env_, None),
                                  _ => (env_, Some(rest)),
                              });
                          });
        } else {
            return Err((env, Error::DecodingError));
        }
    }

    #[inline]
    fn handle_dup(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == DUP {
            match env.pop() {
                None => return Err((env, Error::EmptyStack)),
                Some(v) => {
                    env.push(v);
                    env.push(v);
                    Ok((env, None))
                }
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_swap(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == SWAP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            env.push(a.unwrap());
            env.push(b.unwrap());

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_over(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == OVER {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            env.push(b.unwrap());
            env.push(a.unwrap());
            env.push(b.unwrap());

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_rot(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == ROT {
            let a = env.pop();
            let b = env.pop();
            let c = env.pop();

            if a.is_none() || b.is_none() || c.is_none() {
                return Err((env, Error::EmptyStack));
            }

            env.push(b.unwrap());
            env.push(a.unwrap());
            env.push(c.unwrap());

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_drop(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == DROP {
            match env.pop() {
                None => return Err((env, Error::EmptyStack)),
                _ => Ok((env, None)),
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_depth(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == DEPTH {
            let bytes = BigUint::from(env.stack_size).to_bytes_be();
            let slice = env.alloc(bytes.len());
            for i in 0..bytes.len() {
                slice[i] = bytes[i];
            }
            env.push(slice);
            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_equal(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == EQUALP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            if a1 == b1 {
                env.push(STACK_TRUE);
            } else {
                env.push(STACK_FALSE);
            }

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_ltp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == LTP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            if b1 < a1 {
                env.push(STACK_TRUE);
            } else {
                env.push(STACK_FALSE);
            }

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_gtp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == GTP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            if b1 > a1 {
                env.push(STACK_TRUE);
            } else {
                env.push(STACK_FALSE);
            }

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_concat(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == CONCAT {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            let mut slice = env.alloc(a1.len() + b1.len());

            let mut offset = 0;

            for byte in b1 {
                slice[offset] = *byte;
                offset += 1
            }

            for byte in a1 {
                slice[offset] = *byte;
                offset += 1
            }

            env.push(slice);

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_eval(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == EVAL {
            match env.pop() {
                None => return Err((env, Error::EmptyStack)),
                Some(v) => {
                    Ok((env, Some(Vec::from(v))))
                }
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_times(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == TIMES {
            let count_ = env.pop();

            if count_.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let count = count_.unwrap();

            match env.pop() {
                None => return Err((env, Error::EmptyStack)),
                Some(v) => {
                    let counter = BigUint::from_bytes_be(count);
                    if counter.is_zero() {
                        Ok((env, None))
                    } else {
                        // inject the code itself
                        let mut vec = Vec::from(v);
                        if counter != BigUint::one() {
                            // inject the prefix for the code
                            let mut header = vec![0;offset_by_size(v.len())];
                            write_size_into_slice!(v.len(), header.as_mut_slice());
                            vec.append(&mut header);
                            vec.extend_from_slice(v);
                            // inject the decremented counter
                            let counter = counter.sub(BigUint::one());
                            let mut counter_bytes = counter.to_bytes_be();
                            let mut header =  vec![0;offset_by_size(counter_bytes.len())];
                            write_size_into_slice!(counter_bytes.len(), header.as_mut_slice());
                            vec.append(&mut header);
                            vec.append(&mut counter_bytes);
                            // inject TIMES
                            vec.extend_from_slice(TIMES);
                        }
                        Ok((env, Some(vec)))
                    }
                }
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_set(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == SET {
            match env.pop() {
                None => Err((env, Error::EmptyStack)),
                Some(closure) => {
                    match binparser::word(closure) {
                        nom::IResult::Done(&[0x81, b':', ref rest..], _) => {
                            let word = &closure[0..closure.len() - rest.len() - 2];
                            env.dictionary.insert(word, rest);
                            Ok((env, None))
                        },
                        nom::IResult::Done(&[0x81, b'=', ref rest..], _) => {
                            let word = &closure[0..closure.len() - rest.len() - 2];
                            let mut vec = Vec::new();
                            // inject the code
                            vec.extend_from_slice(rest);
                            // inject [word] \x00SET!
                            let sz = word.len() as u8;
                            if word.len() > 120 {
                                vec.push(121);
                            }
                            vec.push(sz);
                            vec.extend_from_slice(word);
                            vec.extend_from_slice(SET_IMM);
                            Ok((env, Some(vec)))
                        },
                        _ => Err((env, Error::UnknownWord))
                    }
                }
            }
        } else if word == SET_IMM {
            let a = env.pop();
            let b = env.pop();
            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }
            let closure = a.unwrap();
            let val = b.unwrap();
            match binparser::word(closure) {
                nom::IResult::Done(_, _) => {
                    let word = &closure[0..closure.len()];
                    let offset = offset_by_size(val.len());
                    let sz = val.len() + offset;
                    let mut slice = env.alloc(sz);
                    write_size_into_slice!(val.len(), &mut slice);
                    let mut i = offset;
                    for b in val {
                        slice[i] = *b;
                        i += 1;
                    }
                    env.dictionary.insert(word, slice);
                    Ok((env, None))
                },
                _ => Err((env, Error::UnknownWord))
            }
        } else if env.dictionary.contains_key(word) {
            let mut vec = Vec::new();
            {
                let def = env.dictionary.get(word).unwrap();
                vec.extend_from_slice(def);
            }
            Ok((env, Some(vec)))
        } else {
            Err((env, Error::UnknownWord))
        }
    }
}


#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use script::{Env, VM, Error, RequestMessage, ResponseMessage, EnvId, parse, offset_by_size};
    use std::sync::mpsc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use super::binparser;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn env_stack_growth() {
        let mut env = Env::new();
        let target = env.stack.len() * 100;
        for i in 1..target {
            env.push(_EMPTY);
        }
        assert!(env.stack.len() >= target);
    }

    #[test]
    fn env_heap_growth() {
        let mut env = Env::new();
        let sz = env.heap_size;
        for i in 1..100 {
            env.alloc(sz);
        }
        assert!(env.heap_size >= sz * 100);
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
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
            assert_eq!(env.pop(), None);
        });

        eval!("DUP", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });
    }

    #[test]
    fn swap() {
        eval!("0x010203 0x030201 SWAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x030201"));
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
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x030201"));
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
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x010203"));
        });

        eval!("0x00 OVER", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("OVER", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }

    #[test]
    fn depth() {
        eval!("0x010203 0x00 \"Hello\" DEPTH", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("3"));
        });
    }

    #[test]
    fn equal() {
        eval!("0x10 0x20 EQUAL? 0x10 0x10 EQUAL?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn ltgt() {
        eval!("\"a\" \"b\" LT? \"a\" \"a\" LT? \"b\" \"a\" LT?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("\"a\" \"b\" GT? \"a\" \"a\" GT? \"b\" \"a\" GT?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });

    }

    #[test]
    fn concat() {
        eval!("0x10 0x20 CONCAT", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x1020"));
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
    fn times() {
        eval!("0x01 [DUP] 4 TIMES", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("TIMES", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

        eval!("5 TIMES", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });

    }

    #[test]
    fn eval() {
        eval!("[0x01 DUP [DUP] EVAL] EVAL DROP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("EVAL", env, result, {
            assert!(matches!(result.err(), Some(Error::EmptyStack)));
        });
    }

    #[test]
    fn set() {
        eval!("[mydup : DUP DUP] SET 1 mydup mydup", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("1 [current_depth = DEPTH] SET 1 2 3 current_depth", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[mydup DUP DUP] SET 1 mydup mydup", env, result, {
            assert!(result.is_err());
        });

        eval!("SET", env, result, {
            assert!(result.is_err());
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
