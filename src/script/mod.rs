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

use num_bigint::BigUint;
use num_traits::{Zero, One};
use num_traits::ToPrimitive;
use core::ops::{Add, Sub};

use std::collections::BTreeMap;

pub mod envheap;
use self::envheap::EnvHeap;

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
word!(EQUALQ, (a, b => c), b"\x86EQUAL?");
word!(LTQ, (a, b => c), b"\x83LT?");
word!(GTQ, (a, b => c), b"\x83GT?");
word!(LENGTH, (a => b), b"\x86LENGTH");
word!(CONCAT, (a, b => c), b"\x86CONCAT");
word!(SLICE, (a, b, c => d), b"\x85SLICE");
word!(PAD, (a, b, c => d), b"\x83PAD");

// Category: arithmetics
word!(UINT_ADD, (a, b => c), b"\x88UINT/ADD");
word!(UINT_SUB, (a, b => c), b"\x88UINT/SUB");

// Category: Control flow
#[cfg(feature = "scoped_dictionary")]
word!(EVAL_SCOPED, b"\x8BEVAL/SCOPED");
#[cfg(feature = "scoped_dictionary")]
word!(SCOPE_END, b"\x80\x8BEVAL/SCOPED"); // internal word
word!(DOWHILE, b"\x87DOWHILE");
word!(TIMES, b"\x85TIMES");
word!(EVAL, b"\x84EVAL");
word!(EVAL_VALIDP, b"\x8BEVAL/VALID?");
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

// Category: experimental features
word!(FEATUREQ, (a => b), b"\x88FEATURE?");

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
    /// Unparseable remainder
    Superfluous(Vec<u8>)
}
pub mod binparser;
pub use self::binparser::parse as parse_bin;

pub mod textparser;
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
    pub program: Vec<&'a [u8]>,
    stack: Vec<&'a [u8]>,
    stack_size: usize,
    heap: EnvHeap,
    #[cfg(feature = "scoped_dictionary")]
    dictionary: Vec<BTreeMap<&'a [u8], &'a [u8]>>,
    #[cfg(not(feature = "scoped_dictionary"))]
    dictionary: BTreeMap<&'a [u8], &'a [u8]>,
    // current TRY status
    tracking_errors: usize,
    aborting_try: Vec<Error>,
    send_ack: Option<mpsc::Receiver<()>>
}

impl<'a> std::fmt::Debug for Env<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str("Env()")
    }
}

unsafe impl<'a> Send for Env<'a> {}

const _EMPTY: &'static [u8] = b"";

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
        #[cfg(feature = "scoped_dictionary")]
        let dictionary = vec![BTreeMap::new()];
        #[cfg(not(feature = "scoped_dictionary"))]
        let dictionary = BTreeMap::new();
        Ok(Env {
            program: vec![],
            stack: stack,
            stack_size: stack_size,
            heap: EnvHeap::new(HEAP_SIZE),
            dictionary: dictionary,
            tracking_errors: 0,
            aborting_try: Vec::new(),
            send_ack: None
        })
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
        Ok(unsafe { mem::transmute::<& mut [u8], &'a mut [u8]>(self.heap.alloc(len)) })
    }


    #[cfg(feature = "scoped_dictionary")]
    pub fn push_dictionary(&mut self) {
        let dict = self.dictionary.pop().unwrap();
        let new_dict = dict.clone();
        self.dictionary.push(dict);
        self.dictionary.push(new_dict);
    }

    #[cfg(feature = "scoped_dictionary")]
    pub fn pop_dictionary(&mut self) {
        self.dictionary.pop();
        if self.dictionary.len() == 0 {
            self.dictionary.push(BTreeMap::new());
        }
    }
}


use nom;

#[inline]
pub fn offset_by_size(size: usize) -> usize {
    match size {
        0...99 => 1,
        100...255 => 2,
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
    RescheduleEnv(EnvId, Env<'a>, Sender<ResponseMessage<'a>>),
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
pub mod hash;

/// VM is a PumpkinScript scheduler and interpreter. This is the
/// most central part of this module.
///
/// # Example
///
/// ```norun
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

use std::collections::VecDeque;

pub struct VM<'a> {
    inbox: Receiver<RequestMessage<'a>>,
    sender: Sender<RequestMessage<'a>>,
    publisher: pubsub::PublisherAccessor<Vec<u8>>,
    storage: storage::Handler<'a>,
    hlc: timestamp_hlc::Handler<'a>,
    hash: hash::Handler<'a>
}

unsafe impl<'a> Send for VM<'a> {}

type PassResult<'a> = Result<(), Error>;

const STACK_TRUE: &'static [u8] = b"\x01";
const STACK_FALSE: &'static [u8] = b"\x00";

const ERROR_UNKNOWN_WORD: &'static [u8] = b"\x0C\x02";
const ERROR_INVALID_VALUE: &'static [u8] = b"\x0C\x03";
const ERROR_EMPTY_STACK: &'static [u8] = b"\x0C\x04";
const ERROR_DECODING: &'static [u8] = b"\x0C\x05";
const ERROR_DUPLICATE_KEY: &'static [u8] = b"\x0C\x06";
const ERROR_UNKNOWN_KEY: &'static [u8] = b"\x0C\x07";
const ERROR_NO_TX: &'static [u8] = b"\x0C\x08";
const ERROR_DATABASE: &'static [u8] = b"\x0C\x09";

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
            publisher: publisher,
            storage: storage::Handler::new(db_env, db),
            hlc: timestamp_hlc::Handler::new(),
            hash: hash::Handler::new()
        }
    }

    pub fn sender(&self) -> Sender<RequestMessage<'a>> {
        self.sender.clone()
    }

    /// Scheduler. It is supposed to be running in a separate thread
    ///
    /// The scheduler handles all incoming  messages. Once at least one
    /// program is scheduled (`ScheduleEnv`), it will start scheduling work,
    /// after which it will execute one instruction per program at a time.
    /// This way it can execute multiple scripts at the same time.
    ///
    /// Once an environment execution has been terminated, a message will be sent,
    /// depending on the result (`EnvTerminated` or `EnvFailed`)
    pub fn run(&mut self) {
        let mut envs: VecDeque<(EnvId, Env<'a>, Sender<ResponseMessage<'a>>)> = VecDeque::new();

        loop {

            match envs.pop_front() {
                Some((pid, mut env, chan)) => {
                    match self.pass(&mut env, pid.clone()) {
                        Err(Error::Reschedule) => {
                            envs.push_back((pid, env, chan));
                        },
                        Err(err) => {
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         Some(Vec::from(env.stack())),
                                                                         Some(env.stack_size)));
                        }
                        Ok(()) => {
                            if env.program.is_empty() || (env.program.len() == 1 && env.program[0].len() == 0) {
                                let _ = chan.send(ResponseMessage::EnvTerminated(pid,
                                                                                 Vec::from(env.stack()),
                                                                                 env.stack_size));
                            } else {
                                envs.push_back((pid, env, chan));
                            }
                        }
                    };
                },
                None => ()
            }
            let message = if envs.len() == 0 {
                self.inbox.recv()
            } else {
                let msg = self.inbox.try_recv();
                if let Err(mpsc::TryRecvError::Empty) = msg {
                    continue;
                }
                msg.map_err(|_| mpsc::RecvError{})
            };
            match message {
                Err(err) => panic!("error receiving: {:?}", err),
                Ok(RequestMessage::Shutdown) => break,
                Ok(RequestMessage::ScheduleEnv(pid, program, chan)) => {
                    match Env::new() {
                        Ok(mut env) => {
                            match env.alloc(program.len()) {
                                Ok(slice) => {
                                    slice.copy_from_slice(program.as_slice());
                                    env.program.push(slice);
                                    envs.push_back((pid, env, chan));
                                }
                                Err(err) => {
                                    let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                                 err,
                                                                                 None,
                                                                                 None));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         None,
                                                                         None));
                        }
                    }
                }
                Ok(_) => {}
            }
        }
    }

    #[allow(unused_mut)]
    fn pass(&mut self, env: &mut Env<'a>, pid: EnvId) -> PassResult<'a> {
        // Check if this Env has a pending SEND
        if env.send_ack.is_some() {
            match mem::replace(&mut env.send_ack, None) {
                None => (),
                Some(rcvr) =>
                    match rcvr.try_recv() {
                        Err(mpsc::TryRecvError::Empty) => {
                            env.send_ack = Some(rcvr);
                            return Err(Error::Reschedule)
                        },
                        Err(mpsc::TryRecvError::Disconnected) => (),
                        Ok(()) => ()
                    }
            }
        }
        if env.program.len() == 0 {
            return Ok(());
        }
        let program = env.program.pop().unwrap();
        if program.len() == 0 {
            return Ok(());
        }
        if let nom::IResult::Done(rest, data) = binparser::data(program) {
            if env.aborting_try.is_empty() {
                if data.len() == 1 && data[0] <= 10u8 {
                    return handle_error!(env, error_decoding!());
                } else {
                    env.push(&data[offset_by_size(data.len())..]);
                }
            }
            if rest.len() > 0 {
                env.program.push(rest);
            }
            Ok(())
        } else if let nom::IResult::Done(rest, word) = binparser::word_or_internal_word(program) {
            if rest.len() > 0 {
                env.program.push(rest);
            }
            handle_words!(self, env,
                          program,
                          word,
                          new_env,
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
                           self => handle_pad,
                           self => handle_uint_add,
                           self => handle_uint_sub,
                           self => handle_length,
                           self => handle_dowhile,
                           self => handle_times,
                           self => handle_scope_end,
                           self => handle_eval,
                           self => handle_eval_validp,
                           self => handle_eval_scoped,
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
                           // hashing
                           self.hash => handle_hash_sha1,
                           self.hash => handle_hash_sha224,
                           self.hash => handle_hash_sha256,
                           self.hash => handle_hash_sha384,
                           self.hash => handle_hash_sha512,
                           self.hash => handle_hash_sha512_224,
                           self.hash => handle_hash_sha512_256,
                           // pubsub
                           self => handle_send,
                           // features
                           self => handle_featurep,
                           // catch-all (NB: keep it last)
                           self => handle_dictionary
                           })
        } else {
            handle_error!(env, error_decoding!())
        }
    }

    #[inline]
    fn handle_builtins(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if BUILTINS.contains_key(word) {
            let vec = BUILTINS.get(word).unwrap();
            env.program.push(vec.as_slice());
            Ok(())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    fn handle_dup(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DUP);
        let v = stack_pop!(env);

        env.push(v);
        env.push(v);
        Ok(())
    }

    #[inline]
    fn handle_swap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SWAP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_over(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, OVER);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_rot(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, ROT);
        let a = stack_pop!(env);
        let b = stack_pop!(env);
        let c = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_drop(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DROP);
        let _ = stack_pop!(env);

        Ok(())
    }

    #[inline]
    fn handle_depth(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DEPTH);
        let bytes = BigUint::from(env.stack_size).to_bytes_be();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_wrap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
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

        let mut slice = alloc_slice!(size, env);

        let mut offset = 0;
        for item in vec {
            write_size_into_slice!(item.len(), &mut slice[offset..]);
            offset += offset_by_size(item.len());
            slice[offset..offset + item.len()].copy_from_slice(item);
            offset += item.len();
        }
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_equal(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EQUALQ);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if a == b {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_not(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, NOT);
        let a = stack_pop!(env);

        if a == STACK_TRUE {
            env.push(STACK_FALSE);
        } else if a == STACK_FALSE {
            env.push(STACK_TRUE);
        } else {
            return Err(error_invalid_value!(a));
        }

        Ok(())
    }

    #[inline]
    fn handle_and(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, AND);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err(error_invalid_value!(a));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err(error_invalid_value!(b));
        }

        if a == STACK_TRUE && b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else if a == STACK_FALSE || b == STACK_FALSE {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_or(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, OR);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err(error_invalid_value!(a));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err(error_invalid_value!(b));
        }

        if a == STACK_TRUE || b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_ifelse(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, IFELSE);
        let else_ = stack_pop!(env);
        let then = stack_pop!(env);
        let cond = stack_pop!(env);

        if cond == STACK_TRUE {
            env.program.push(then);
            Ok(())
        } else if cond == STACK_FALSE {
            env.program.push(else_);
            Ok(())
        } else {
            Err(error_invalid_value!(cond))
        }
    }

    #[inline]
    fn handle_ltp(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, LTQ);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if b < a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_gtp(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, GTQ);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        if b > a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_concat(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, CONCAT);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let slice = alloc_slice!(a.len() + b.len(), env);

        slice[0..b.len()].copy_from_slice(b);
        slice[b.len()..b.len()+a.len()].copy_from_slice(a);

        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_slice(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SLICE);
        let end = stack_pop!(env);
        let start = stack_pop!(env);
        let slice = stack_pop!(env);

        let start_int = BigUint::from_bytes_be(start).to_u64().unwrap() as usize;
        let end_int = BigUint::from_bytes_be(end).to_u64().unwrap() as usize;

        // range conditions
        if start_int > end_int {
            return Err(error_invalid_value!(start));
        }

        if start_int > slice.len() - 1 {
            return Err(error_invalid_value!(start));
        }

        if end_int > slice.len() {
            return Err(error_invalid_value!(end));
        }

        env.push(&slice[start_int..end_int]);

        Ok(())
    }

    #[inline]
    fn handle_pad(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, PAD);
        let byte = stack_pop!(env);
        let size = stack_pop!(env);
        let value = stack_pop!(env);

        if byte.len() != 1 {
            return Err(error_invalid_value!(byte));
        }

        let size_int = BigUint::from_bytes_be(size).to_u64().unwrap() as usize;

        if size_int > 1024 {
            return Err(error_invalid_value!(size));
        }

        let slice = alloc_slice!(size_int, env);

        for i in 0..size_int-value.len() {
            slice[i] = byte[0];
        }
        slice[size_int-value.len()..].copy_from_slice(value);

        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_length(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, LENGTH);
        let a = stack_pop!(env);

        let len = BigUint::from(a.len() as u64);
        let len_bytes = len.to_bytes_be();

        let slice = alloc_and_write!(len_bytes.as_slice(), env);

        env.push(slice);

        Ok(())
    }

    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_eval_scoped(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EVAL_SCOPED);
        env.push_dictionary();
        let a = stack_pop!(env);
        env.program.push(SCOPE_END);
        env.program.push(a);
        Ok(())
    }

    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_eval_scoped(&mut self, _: &Env<'a>, _: &'a [u8], _: EnvId) -> PassResult<'a> {
        Err(Error::UnknownWord)
    }


    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_scope_end(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SCOPE_END);
        env.pop_dictionary();
        Ok(())
    }


    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_scope_end(&mut self, _: &mut Env<'a>, _: &'a [u8], _: EnvId) -> PassResult<'a> {
        Err(Error::UnknownWord)
    }

    #[inline]
    fn handle_uint_add(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UINT_ADD);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_uint = BigUint::from_bytes_be(a);
        let b_uint = BigUint::from_bytes_be(b);

        let c_uint = a_uint.add(b_uint);

        let c_bytes = c_uint.to_bytes_be();

        let slice = alloc_and_write!(c_bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_uint_sub(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UINT_SUB);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_uint = BigUint::from_bytes_be(a);
        let b_uint = BigUint::from_bytes_be(b);

        if a_uint > b_uint {
            return Err(error_invalid_value!(a));
        }

        let c_uint = b_uint.sub(a_uint);

        let c_bytes = c_uint.to_bytes_be();
        let slice = alloc_and_write!(c_bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }


    #[inline]
    fn handle_eval(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EVAL);
        let a = stack_pop!(env);
        env.program.push(a);
        Ok(())
    }

    #[inline]
    fn handle_eval_validp(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, EVAL_VALIDP);
        let a = stack_pop!(env);
        if a.len() == 1 && a[0] <= 10u8 {
            env.push(STACK_FALSE);
        } else if parse_bin(a).is_ok() {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }
        Ok(())
    }

    #[inline]
    fn handle_try(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY);
        let v = stack_pop!(env);
        env.tracking_errors += 1;
        env.program.push(TRY_END);
        env.program.push(v);
        Ok(())
    }

    #[inline]
    fn handle_try_end(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY_END);
        env.tracking_errors -= 1;
        if env.aborting_try.is_empty() {
            env.push(_EMPTY);
            Ok(())
        } else if let Some(Error::ProgramError(err)) = env.aborting_try.pop() {
            let slice = alloc_and_write!(err.as_slice(), env);
            env.push(slice);
            Ok(())
        } else {
            env.push(_EMPTY);
            Ok(())
        }
    }

    #[inline]
    fn handle_unwrap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UNWRAP);
        let mut current = stack_pop!(env);
        while current.len() > 0 {
            match binparser::data(current) {
                nom::IResult::Done(rest, val) => {
                    env.push(&val[offset_by_size(val.len())..]);
                    current = rest
                },
                _ => {
                    return Err(error_invalid_value!(current))
                }
            }
        }
        Ok(())
    }

    #[inline]
    fn handle_dowhile(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DOWHILE);
        let v = stack_pop!(env);

        let mut vec = Vec::new();

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

        let slice = alloc_and_write!(vec.as_slice(), env);
        env.program.push(slice);
        env.program.push(v);

        Ok(())
    }

    #[inline]
    fn handle_times(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TIMES);
        let count = stack_pop!(env);

        let v = stack_pop!(env);

        let counter = BigUint::from_bytes_be(count);
        if counter.is_zero() {
            Ok(())
        } else {
            let mut vec = Vec::new();
            if counter != BigUint::one() {
                // inject the prefix for the code
                let mut header = vec![0;offset_by_size(v.len())];
                write_size_into_slice!(v.len(), header.as_mut_slice());
                vec.append(&mut header);
                vec.extend_from_slice(v);
                // inject the decremented counter
                let counter = counter.sub(BigUint::one());
                let mut counter_bytes = counter.to_bytes_be();
                let mut header = vec![0;offset_by_size(counter_bytes.len())];
                write_size_into_slice!(counter_bytes.len(), header.as_mut_slice());
                vec.append(&mut header);
                vec.append(&mut counter_bytes);
                // inject TIMES
                vec.extend_from_slice(TIMES);
            }
            let slice = alloc_and_write!(vec.as_slice(), env);
            env.program.push(slice);
            env.program.push(v);
            Ok(())
        }
    }

    #[inline]
    fn handle_set(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SET);
        let word = stack_pop!(env);
        let value = stack_pop!(env);
        match binparser::word(word) {
            nom::IResult::Done(_, _) => {
                let slice = alloc_slice!(value.len() + offset_by_size(value.len()), env);
                write_size_into_slice!(value.len(), slice);
                let offset = offset_by_size(value.len());
                slice[offset..offset + value.len()].copy_from_slice(value);
                #[cfg(feature = "scoped_dictionary")]
                {
                    let mut dict = env.dictionary.pop().unwrap();
                    dict.insert(word, slice);
                    env.dictionary.push(dict);
                }
                #[cfg(not(feature = "scoped_dictionary"))]
                env.dictionary.insert(word, slice);
                Ok(())
            },
            _ => Err(error_invalid_value!(word))
        }
    }

    fn handle_def(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DEF);
        let word = stack_pop!(env);
        let value = stack_pop!(env);
        match binparser::word(word) {
            nom::IResult::Done(_, _) => {
                #[cfg(feature = "scoped_dictionary")]
                {
                    let mut dict = env.dictionary.pop().unwrap();
                    dict.insert(word, value);
                    env.dictionary.push(dict);
                }
                #[cfg(not(feature = "scoped_dictionary"))]
                env.dictionary.insert(word, value);
                Ok(())
            },
            _ => Err(error_invalid_value!(word))
        }
    }


    #[inline]
    fn handle_send(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SEND);
        let topic = stack_pop!(env);
        let data = stack_pop!(env);

        let receiver = self.publisher.send_async(Vec::from(topic), Vec::from(data));

        env.send_ack = Some(receiver);

        Ok(())
    }

    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_dictionary(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        let dict = env.dictionary.pop().unwrap();
        if dict.contains_key(word) {
            {
                let def = dict.get(word).unwrap();
                env.program.push(def);
            }
            env.dictionary.push(dict);
            Ok(())
        } else {
            env.dictionary.push(dict);
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_dictionary(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if env.dictionary.contains_key(word) {
            {
                let def = env.dictionary.get(word).unwrap();
                env.program.push(def);
            }
            Ok(())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    #[allow(unused_variables)]
    fn handle_featurep(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, FEATUREQ);
        let name = stack_pop!(env);

        #[cfg(feature = "scoped_dictionary")]
        {
            if name == "scoped_dictionary".as_bytes() {
                env.push(STACK_TRUE);
                return Ok(())
            }
        }

        env.push(STACK_FALSE);

        Ok(())
    }

}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use script::{Env, VM, Error, RequestMessage, ResponseMessage, EnvId, parse, offset_by_size};
    use std::sync::mpsc;
    use std::fs;
    use std::thread;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use super::binparser;
    use pubsub;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn error_macro() {
        if let Error::ProgramError(err) = error_program!("Test".as_bytes(), "123".as_bytes(), b"\x0C\x33") {
            assert_eq!(err, parsed_data!("[\"Test\" [\"123\"] 0x33]"));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn env_stack_growth() {
        let mut env = Env::new().unwrap();
        let target = env.stack.len() * 100;
        for i in 1..target {
            env.push(_EMPTY);
        }
        assert!(env.stack.len() >= target);
    }

    use std::time::Duration;

    #[test]
    fn send() {
        eval!("\"Hello\" \"Topic\" SEND", env, result, publisher_accessor, {
            let (sender1, receiver1) = mpsc::channel();
            publisher_accessor.subscribe(Vec::from("Topic"), sender1);

            let (sender0, receiver0) = mpsc::channel();
            thread::spawn(move ||  {
               match receiver1.recv() {
                  Ok((topic, message, callback)) => {
                     callback.send(());
                     sender0.send((topic, message));
                  },
                  e => panic!("unexpected result {:?}", e)
               };

            });

        }, {
            assert!(!result.is_err());

            let result = receiver0.recv_timeout(Duration::from_secs(1)).unwrap();
            assert_eq!(result, (Vec::from("Topic"), Vec::from("Hello")));
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
            assert_error!(result, "[\"Unknown word: NOTAWORD\" ['NOTAWORD] 2]");
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
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Unknown word: NOTAWORD\" ['NOTAWORD] 2]"));
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

        eval!("1 TRY", env, result, {
            println!("res: {:?}", env);
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Decoding error\" [] 5]"));
            assert_eq!(env.pop(), None);
        });

    }

    use test::Bencher;

    #[bench]
    fn times(b: &mut Bencher) {
       bench_eval!("[1 DROP] 1000 TIMES", b);
    }

    #[bench]
    fn ackermann(b: &mut Bencher) { // HT @5HT
        bench_eval!("['n SET 'm SET m 0 EQUAL? [n 1 UINT/ADD] \
        [n 0 EQUAL? [m 1 UINT/SUB 1 ack] [m 1 UINT/SUB m n 1 UINT/SUB ack ack] IFELSE] IFELSE] \
        'ack DEF \
        3 4 ack", b);
    }



}
