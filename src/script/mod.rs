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
use num_traits::ToPrimitive;
use core::ops::Sub;
use std::cmp;

use std::collections::BTreeMap;

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
word!(UNWRAP, b"\x86UNWRAP");
word!(WRAP, b"\x84WRAP");

// Category: Byte arrays
word!(EQUALP, (a, b => c), b"\x86EQUAL?");
word!(LTP, (a, b => c), b"\x83LT?");
word!(GTP, (a, b => c), b"\x83GT?");
word!(LENGTH, (a => b), b"\x86LENGTH");
word!(CONCAT, (a, b => c), b"\x86CONCAT");
word!(SLICE, (a, b, c => d), b"\x85SLICE");

// Category: Control flow
word!(DOWHILE, b"\x87DOWHILE");
word!(TIMES, b"\x85TIMES");
word!(EVAL, b"\x84EVAL");
word!(TRY, b"\x83TRY");
word!(TRY_END, b"\x80\x83TRY"); // internal word
word!(SET, b"\x83SET");
word!(DEF, b"\x83DEF");
word!(IF, b"\x82IF"); // for reference, implemented in builtins
word!(IFELSE, b"\x86IFELSE");

// Category: Logical operations
word!(NOT, (a => c), b"\x83NOT");
word!(AND, (a, b => c), b"\x83AND");
word!(OR, (a, b => c), b"\x82OR");

// Category: pubsub
word!(SEND, (a => ), b"\x84SEND");

use std::str;

// Builtin words that are implemented in PumpkinScript
lazy_static! {
  static ref BUILTIN_FILE: &'static [u8] = include_bytes!("builtins");

  static ref BUILTIN_DEFS: Vec<Vec<u8>> = textparser::programs(*BUILTIN_FILE).unwrap().1;

  static ref BUILTINS: BTreeMap<&'static [u8], Vec<u8>> = {
      let mut map = BTreeMap::new();
      let ref defs : Vec<Vec<u8>> = *BUILTIN_DEFS;
      for definition in defs {
          match binparser::word(definition.as_slice()) {
              nom::IResult::Done(&[0x81, b':', ref rest..], _) => {
                  let word = &definition[0..definition.len() - rest.len() - 2];
                  map.insert(word, Vec::from(rest));
              },
              other => panic!("builtin definition parse error {:?}", other)
          }
      }
      map
  };
}

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
    /// Word is unknown
    UnknownWord,
    /// An internal scheduler's error to indicate that currently
    /// executed environment should be rescheduled from the same point
    Reschedule,
    /// Program Error
    ProgramError(Vec<u8>),
    /// Unable to (re)allocate the heap so the returning slice points to
    /// unallocated memory.
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
pub mod binparser;
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
    dictionary: BTreeMap<&'a [u8], &'a [u8]>,
    // current TRY status
    tracking_errors: usize,
    aborting_try: Vec<Error>
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
                    dictionary: BTreeMap::new(),
                    tracking_errors: 0,
                    aborting_try: Vec::new()
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
            match unsafe {
                heap::reallocate(self.heap,
                                 self.heap_size,
                                 self.heap_size + increase,
                                 self.heap_align).as_mut()
            } {
                Some(heap) => {
                    self.heap = heap;
                    self.heap_size = self.heap_size + increase;
                }
                None => return Err(Error::HeapAllocFailed)
            }
        }
        let mut space = unsafe { slice::from_raw_parts_mut(self.heap, self.heap_size) };
        let slice = &mut space[self.heap_ptr..self.heap_ptr + len];
        self.heap_ptr += len;
        Ok(slice)
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

use pubsub;

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
    publisher: pubsub::PublisherAccessor<Vec<u8>>,
    storage: storage::Handler<'a>,
    hlc: timestamp_hlc::Handler<'a>,
}

unsafe impl<'a> Send for VM<'a> {}

type PassResult<'a> = Result<(Env<'a>, Option<Vec<u8>>), (Env<'a>, Error)>;

const STACK_TRUE: &'static [u8] = b"\x01";
const STACK_FALSE: &'static [u8] = b"\x00";

const ERROR_UNKNOWN_WORD: &'static [u8] = b"\x01\x02";
const ERROR_INVALID_VALUE: &'static [u8] = b"\x01\x03";
const ERROR_EMPTY_STACK: &'static [u8] = b"\x01\x04";
const ERROR_DECODING: &'static [u8] = b"\x01\x05";
const ERROR_DUPLICATE_KEY: &'static [u8] = b"\x01\x06";
const ERROR_UNKNOWN_KEY: &'static [u8] = b"\x01\x07";
const ERROR_NO_TX: &'static [u8] = b"\x01\x08";
const ERROR_DATABASE: &'static [u8] = b"\x01\x09";

impl<'a> VM<'a> {
    /// Creates an instance of VM with three communication channels:
    ///
    /// * Response sender
    /// * Internal sender
    /// * Request receiver
    pub fn new(db_env: &'a lmdb::Environment, db: &'a lmdb::Database<'a>,
               publisher: pubsub::PublisherAccessor<Vec<u8>>) -> Self {
        let (sender, receiver) = mpsc::channel::<RequestMessage<'a>>();
        VM {
            inbox: receiver,
            sender: sender.clone(),
            loopback: sender.clone(),
            publisher: publisher,
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
        if program.len() == 0 {
            return Ok((env, None));
        }
        let slice0 = env.alloc(program.len());
        if slice0.is_err() {
            return Err((env, slice0.unwrap_err()));
        }
        let mut slice = slice0.unwrap();
        for i in 0..program.len() {
            slice[i] = program[i];
        }
        if let nom::IResult::Done(_, data) = binparser::data(slice) {
            if env.aborting_try.is_empty() {
                env.push(&data[offset_by_size(data.len())..]);
            }
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
                          {self => handle_builtins,
                           self => handle_drop,
                           self => handle_dup,
                           self => handle_swap,
                           self => handle_rot,
                           self => handle_over,
                           self => handle_depth,
                           self => handle_wrap,
                           self => handle_ltp,
                           self => handle_gtp,
                           self => handle_equal,
                           self => handle_concat,
                           self => handle_slice,
                           self => handle_length,
                           self => handle_dowhile,
                           self => handle_times,
                           self => handle_eval,
                           self => handle_try,
                           self => handle_try_end,
                           self => handle_unwrap,
                           self => handle_set,
                           self => handle_def,
                           self => handle_not,
                           self => handle_and,
                           self => handle_or,
                           self => handle_ifelse,
                           // storage
                           self.storage => handle_write,
                           self.storage => handle_read,
                           self.storage => handle_assoc,
                           self.storage => handle_assocq,
                           self.storage => handle_retr,
                           self.storage => handle_commit,
                           self.storage => handle_cursor,
                           self.storage => handle_cursor_first,
                           self.storage => handle_cursor_next,
                           self.storage => handle_cursor_prev,
                           self.storage => handle_cursor_last,
                           self.storage => handle_cursor_seek,
                           self.storage => handle_cursor_cur,
                           // timestamping
                           self.hlc => handle_hlc,
                           self.hlc => handle_hlc_lc,
                           self.hlc => handle_hlc_tick,
                           self.hlc => handle_hlc_ltp,
                           self.hlc => handle_hlc_gtp,
                           // pubsub
                           self => handle_send,
                           // catch-all (NB: keep it last)
                           self => handle_dictionary
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
            return Err((env, error_decoding!()));
        }
    }

    #[inline]
    fn handle_builtins(&mut self, env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if BUILTINS.contains_key(word) {
            let vec = BUILTINS.get(word).unwrap().clone();
            Ok((env, Some(vec)))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    fn handle_dup(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DUP);
        let v = stack_pop!(env);

        env.push(v);
        env.push(v);
        Ok((env, None))
    }

    #[inline]
    fn handle_swap(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SWAP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(a);
        env.push(b);

        Ok((env, None))
    }

    #[inline]
    fn handle_over(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, OVER);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(b);

        Ok((env, None))
    }

    #[inline]
    fn handle_rot(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, ROT);
        let a = stack_pop!(env);
        let b = stack_pop!(env);
        let c = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(c);

        Ok((env, None))
    }

    #[inline]
    fn handle_drop(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DROP);
        let _ = stack_pop!(env);

        Ok((env, None))
    }

    #[inline]
    fn handle_depth(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DEPTH);
        let bytes = BigUint::from(env.stack_size).to_bytes_be();
        let slice0 = env.alloc(bytes.len());
        if slice0.is_err() {
            return Err((env, slice0.unwrap_err()));
        }
        let mut slice = slice0.unwrap();
        for i in 0..bytes.len() {
            slice[i] = bytes[i];
        }
        env.push(slice);
        Ok((env, None))
    }

    #[inline]
    fn handle_wrap(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, WRAP);
        let n = stack_pop!(env);

        let mut n_int = BigUint::from_bytes_be(n).to_u64().unwrap() as usize;

        let mut vec = Vec::new();

        while n_int > 0 {
            let item = stack_pop!(env);
            vec.insert(0, item);
            n_int -= 1;
        }

        let size = vec.clone().into_iter()
            .fold(0, |a, item| a + item.len() + offset_by_size(item.len()));

        match env.alloc(size) {
            Ok(mut slice) => {
                let mut offset = 0;
                for item in vec {
                    write_size_into_slice!(item.len(), &mut slice[offset..]);
                    offset += offset_by_size(item.len());
                    for b in item {
                        slice[offset] = *b;
                        offset += 1;
                    }
                }
                env.push(slice);
            }
            Err(err) => return Err((env, err))
        }
        Ok((env, None))
    }

    #[inline]
    fn handle_equal(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EQUALP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if a == b {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_not(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, NOT);
        let a = stack_pop!(env);

        if a == STACK_TRUE {
            env.push(STACK_FALSE);
        } else if a == STACK_FALSE {
            env.push(STACK_TRUE);
        } else {
            return Err((env, error_invalid_value!(a)));
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_and(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, AND);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err((env, error_invalid_value!(a)));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err((env, error_invalid_value!(b)));
        }

        if a == STACK_TRUE && b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else if a == STACK_FALSE || b == STACK_FALSE {
            env.push(STACK_FALSE);
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_or(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, OR);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err((env, error_invalid_value!(a)));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err((env, error_invalid_value!(b)));
        }

        if a == STACK_TRUE || b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_ifelse(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, IFELSE);
        let else_ = stack_pop!(env);
        let then = stack_pop!(env);
        let cond = stack_pop!(env);

        if cond == STACK_TRUE {
            Ok((env, Some(Vec::from(then))))
        } else if cond == STACK_FALSE {
            Ok((env, Some(Vec::from(else_))))
        } else {
            Err((env, error_invalid_value!(cond)))
        }
    }

    #[inline]
    fn handle_ltp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, LTP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if b < a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_gtp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, GTP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if b > a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok((env, None))
    }

    #[inline]
    fn handle_concat(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, CONCAT);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let slice0 = env.alloc(a.len() + b.len());
        if slice0.is_err() {
            return Err((env, slice0.unwrap_err()));
        }
        let mut slice = slice0.unwrap();
        let mut offset = 0;

        for byte in b {
            slice[offset] = *byte;
            offset += 1
        }

        for byte in a {
            slice[offset] = *byte;
            offset += 1
        }

        env.push(slice);

        Ok((env, None))
    }

    #[inline]
    fn handle_slice(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SLICE);
        let end = stack_pop!(env);
        let start = stack_pop!(env);
        let slice = stack_pop!(env);

        let start_int = BigUint::from_bytes_be(start).to_u64().unwrap() as usize;
        let end_int = BigUint::from_bytes_be(end).to_u64().unwrap() as usize;

        // range conditions
        if start_int > end_int {
            return Err((env, error_invalid_value!(start)));
        }

        if start_int > slice.len() - 1 {
            return Err((env, error_invalid_value!(start)));
        }

        if end_int > slice.len() {
            return Err((env, error_invalid_value!(end)));
        }

        env.push(&slice[start_int..end_int]);

        Ok((env, None))
    }

    #[inline]
    fn handle_length(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, LENGTH);
        let a = stack_pop!(env);

        let len = BigUint::from(a.len() as u64);
        let len_bytes = len.to_bytes_be();

        let slice0 = env.alloc(len_bytes.len());
        if slice0.is_err() {
            return Err((env, slice0.unwrap_err()));
        }
        let mut slice = slice0.unwrap();

        let mut offset = 0;

        for byte in len_bytes {
            slice[offset] = byte;
            offset += 1
        }

        env.push(slice);

        Ok((env, None))
    }

    #[inline]
    fn handle_eval(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EVAL);
        let a = stack_pop!(env);
        Ok((env, Some(Vec::from(a))))
    }

    #[inline]
    fn handle_try(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY);
        let v = stack_pop!(env);
        env.tracking_errors += 1;
        let mut vec = Vec::from(v);
        vec.extend_from_slice(TRY_END);
        Ok((env, Some(vec)))
    }

    #[inline]
    fn handle_try_end(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY_END);
        env.tracking_errors -= 1;
        if env.aborting_try.is_empty() {
            env.push(_EMPTY);
            Ok((env, None))
        } else if let Some(Error::ProgramError(err)) = env.aborting_try.pop() {
            match env.alloc(err.len()) {
                Ok(slice) => {
                    let mut i = 0;
                    for byte in err {
                        slice[i] = byte;
                        i += 1;
                    }
                    env.push(slice);
                    Ok((env, None))
                }
                Err(err) => Err((env, err))
            }
        } else {
            env.push(_EMPTY);
            Ok((env, None))
        }
    }

    #[inline]
    fn handle_unwrap(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UNWRAP);
        let mut current = stack_pop!(env);
        while current.len() > 0 {
            match binparser::data(current) {
                nom::IResult::Done(rest, val) => {
                    env.push(&val[offset_by_size(val.len())..]);
                    current = rest
                },
                _ => {
                    return Err((env, error_invalid_value_word!(current)))
                }
            }
        }
        Ok((env, None))
    }

    #[inline]
    fn handle_dowhile(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DOWHILE);
        let v = stack_pop!(env);

        // inject the code itself
        let mut vec = Vec::from(v);

        let mut header = vec![0;offset_by_size(v.len() + DOWHILE.len() + offset_by_size(v.len()))];
        write_size_into_slice!(offset_by_size(v.len()) + v.len() + DOWHILE.len(), header.as_mut_slice());
        vec.append(&mut header);

        // inject code closure size
        let mut header = vec![0;offset_by_size(v.len())];
        write_size_into_slice!(v.len(), header.as_mut_slice());
        vec.append(&mut header);

        // inject code closure
        vec.extend_from_slice(v);
        // inject DOWHILE
        vec.extend_from_slice(DOWHILE);
        // inject IF
        vec.extend_from_slice(IF);

        Ok((env, Some(vec)))
    }

    #[inline]
    fn handle_times(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TIMES);
        let count = stack_pop!(env);

        let v = stack_pop!(env);

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

    #[inline]
    fn handle_set(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SET);
        let word = stack_pop!(env);
        let value = stack_pop!(env);
        match binparser::word(word) {
            nom::IResult::Done(_, _) => {
                match env.alloc(value.len() + offset_by_size(value.len())) {
                    Ok(mut slice) => {
                        write_size_into_slice!(value.len(), slice);
                        let mut offset = offset_by_size(value.len());
                        for b in value {
                            slice[offset] = *b;
                            offset += 1;
                        }
                        env.dictionary.insert(word, slice);
                        Ok((env, None))
                    }
                    Err(err) => return Err((env, err))
                }
            },
            _ => Err((env, error_invalid_value!(word)))
        }
    }

    fn handle_def(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DEF);
        let word = stack_pop!(env);
        let value = stack_pop!(env);
        match binparser::word(word) {
            nom::IResult::Done(_, _) => {
                env.dictionary.insert(word, value);
                Ok((env, None))
            },
            _ => Err((env, error_invalid_value!(word)))
        }
    }


    #[inline]
    fn handle_send(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SEND);
        let topic = stack_pop!(env);
        let data = stack_pop!(env);

        self.publisher.send(Vec::from(topic), Vec::from(data));

        Ok((env, None))
    }

    #[inline]
    fn handle_dictionary(&mut self, env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if env.dictionary.contains_key(word) {
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
    use pubsub;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn env_stack_growth() {
        let mut env = Env::new().unwrap();
        let target = env.stack.len() * 100;
        for i in 1..target {
            env.push(_EMPTY);
        }
        assert!(env.stack.len() >= target);
    }

    #[test]
    fn env_heap_growth() {
        let mut env = Env::new().unwrap();
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn _2dup() {
        eval!("1 2 2DUP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("2"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("1"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("2"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("1"));
            assert_eq!(env.pop(), None);
        });
        eval!("2 2DUP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
        eval!("2DUP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn _2drop() {
        eval!("1 2 2DROP", env, {
            assert_eq!(env.pop(), None);
        });
        eval!("2 2DROP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
        eval!("2DROP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn if_() {
        eval!("0x01 [0x20] IF 0x00 [0x30] IF", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x20"));
            assert_eq!(env.pop(), None);
            assert_eq!(env.pop(), None);
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("0x10 SWAP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("0x010203 ROT", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("ROT", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("OVER", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
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
    fn not() {
        eval!("0x10 0x20 EQUAL? NOT 0x10 0x10 EQUAL? NOT", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn and() {
        eval!("0x01 0x01 AND 0x00 0x01 AND 0x01 0x00 AND 0x00 0x00 AND", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn or() {
        eval!("0x01 0x01 OR 0x00 0x01 OR 0x01 0x00 OR 0x00 0x00 OR", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn ifelse() {
        eval!("0x01 [0x10] [0x20] IFELSE 0x00 [0x10] [0x20] IFELSE", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x20"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x10"));
            assert_eq!(env.pop(), None);
        });

        eval!("0x10 [0x10] [0x20] IFELSE", env, result, {
            assert_error!(result, "[\"Invalid value\" [16] 3]");
        });

        eval!("[0x10] [0x20] IFELSE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("[0x20] IFELSE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("IFELSE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn concat() {
        eval!("0x10 0x20 CONCAT", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x1020"));
            assert_eq!(env.pop(), None);
        });

        eval!("0x20 CONCAT", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("CONCAT", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn slice() {
        eval!("\"Hello\" 0 5 SLICE", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\""));
            assert_eq!(env.pop(), None);
        });

        eval!("\"Hello\" 1 4 SLICE", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"ell\""));
            assert_eq!(env.pop(), None);
        });


        eval!("\"Hello\" 0 0 SLICE", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"\""));
            assert_eq!(env.pop(), None);
        });

        eval!("\"Hello\" 4 2 SLICE", env, result, {
            assert_error!(result, "[\"Invalid value\" [4] 3]");
        });

        eval!("\"Hello\" 5 7 SLICE", env, result, {
            assert_error!(result, "[\"Invalid value\" [5] 3]");
        });

        eval!("\"Hello\" 1 7 SLICE", env, result, {
            assert_error!(result, "[\"Invalid value\" [7] 3]");
        });

        eval!("0 1 SLICE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("1 SLICE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("SLICE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

    }

    #[test]
    fn length() {
        eval!("[] LENGTH 0x10 LENGTH", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });

        eval!("LENGTH", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("5 TIMES", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

    }

    #[test]
    fn dowhile() {
        eval!("1 2 3 [1 EQUAL? NOT] DOWHILE DEPTH", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });

        eval!("1 2 3 [1 EQUAL?] DOWHILE DEPTH", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x02"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x02"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("[0] DOWHILE", env, {
            assert_eq!(env.pop(), None);
        });

        eval!("DOWHILE", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("[100] DOWHILE", env, result, {
            assert_error!(result, "[\"Invalid value\" [100] 3]");
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
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn unwrap() {
        eval!("[1 2] UNWRAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x02"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("[] UNWRAP", env, {
            assert_eq!(env.pop(), None);
        });

        eval!("[1 DUP] UNWRAP", env, result, {
            assert_error!(result, "[\"Invalid value\" [DUP] 3]");
        });

        eval!("UNWRAP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn stack() {
        eval!("1 2 3 STACK", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[1 2 3]"));
            assert_eq!(env.pop(), None);
        });
        eval!("1 2 3 STACK UNWRAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("3"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("2"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("1"));
            assert_eq!(env.pop(), None);
        });
        eval!("STACK", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[]"));
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn wrap() {
        eval!("1 2 3 2 WRAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[2 3]"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("1"));
            assert_eq!(env.pop(), None);
        });
        eval!("1 2 3 2 WRAP UNWRAP", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("3"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("2"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("1"));
            assert_eq!(env.pop(), None);
        });
        eval!("1 2 WRAP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("WRAP", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn someq() {
        eval!("[1 2] SOME?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("[] SOME?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });

        eval!("SOME?", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn noneq() {
        eval!("[1 2] NONE?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
            assert_eq!(env.pop(), None);
        });

        eval!("[] NONE?", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("NONE?", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn set() {
        eval!("1 DEPTH 'current_depth SET 1 2 3 current_depth", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[DUP DUP] 'mydup SET mydup", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[DUP DUP]"));
            assert_eq!(env.pop(), None);
        });

        eval!("0 0 SET", env, result, {
            assert_error!(result, "[\"Invalid value\" [0] 3]");
        });

        eval!("'a SET", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("SET", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

    }

    #[test]
    fn def() {
        eval!("[DUP DUP] 'mydup DEF 1 mydup mydup", env, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("[DUP DUP] 0 DEF 1 mydup mydup", env, result, {
            assert_error!(result, "[\"Invalid value\" [0] 3]");
        });

        eval!("'a DEF", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("DEF", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

    }

    #[test]
    fn invalid_eval() {
        eval!("0x10 EVAL", env, result, {
            assert_error!(result, "[\"Decoding error\" [] 5]");
        });
    }

    use std::time::Duration;

    #[test]
    fn send() {
        eval!("\"Hello\" \"Topic\" SEND", env, result, publisher_accessor, {
            let (sender1, receiver1) = mpsc::channel();
            publisher_accessor.subscribe(Vec::from("Topic"), sender1);
            let (sender2, receiver2) = mpsc::channel();
            publisher_accessor.subscribe(Vec::from("Topic"), sender2);

        }, {
            assert!(!result.is_err());
            assert_eq!(receiver1.recv_timeout(Duration::from_secs(1)).unwrap(), (Vec::from("Topic"), Vec::from("Hello")));
            assert_eq!(receiver2.recv_timeout(Duration::from_secs(1)).unwrap(), (Vec::from("Topic"), Vec::from("Hello")));
        });

        eval!("\"Hello\" \"Topic1\" SEND", env, result, publisher_accessor, {
            let (sender, receiver) = mpsc::channel();
            publisher_accessor.subscribe(Vec::from("Topic"), sender);
        }, {
            assert!(!result.is_err());
            assert!(receiver.recv_timeout(Duration::from_secs(1)).is_err());
        });

        eval!("\"Topic\" SEND", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });

        eval!("SEND", env, result, {
            assert_error!(result, "[\"Empty stack\" [] 4]");
        });
    }

    #[test]
    fn unknown_word() {
        eval!("NOTAWORD", env, result, {
            assert_error!(result, "[\"Unknown word: NOTAWORD\" [NOTAWORD] 2]");
        });
    }

    #[test]
    fn nothing() {
        eval!("", env, {
            assert_eq!(env.pop(), None);
        });
    }

    #[test]
    fn try() {
        eval!("[1 DUP] TRY", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[]"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });

        eval!("[DUP] TRY", env, result, {
            assert!(!result.is_err());
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Empty stack\" [] 4]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[NOTAWORD] TRY", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Unknown word: NOTAWORD\" [NOTAWORD] 2]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[[DUP] TRY 0x20 NOT] TRY", env, result, {
            assert!(!result.is_err());
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Invalid value\" [0x20] 3]"));
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Empty stack\" [] 4]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[1 DUP] TRY STACK DROP DUP", env, result, {
            assert!(result.is_err());
        });

        eval!("[DUP] TRY STACK DROP DUP", env, result, {
            assert!(result.is_err());
        });

    }

}
