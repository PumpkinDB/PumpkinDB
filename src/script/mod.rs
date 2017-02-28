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

word!(TRY, b"\x83TRY");
word!(TRY_END, b"\x80\x83TRY"); // internal word


use std::str;

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

    /// Returns a copy of the entire stack
    #[inline]
    pub fn stack_copy(&self) -> Vec<Vec<u8>> {
        self.stack.clone().into_iter().map(|v| Vec::from(v)).collect()
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
pub enum RequestMessage {
    /// Requests scheduling a new environment with a given
    /// id and a program.
    ScheduleEnv(EnvId, Vec<u8>, Sender<ResponseMessage>),
    /// Requests VM shutdown
    Shutdown,
}

/// Messages received from the [VM](struct.VM.html) thread.
#[derive(Debug)]
pub enum ResponseMessage {
    /// Notifies of successful environment termination with
    /// an id, stack and top of the stack pointer.
    EnvTerminated(EnvId, Vec<Vec<u8>>, usize),
    /// Notifies of abnormal environment termination with
    /// an id, error, stack and top of the stack pointer.
    EnvFailed(EnvId, Error, Option<Vec<Vec<u8>>>, Option<usize>),
}

pub type TrySendError<T> = std::sync::mpsc::TrySendError<T>;

use lmdb;

use pubsub;

pub mod core;
pub mod stack;
pub mod numbers;
pub mod binaries;
pub mod storage;
pub mod timestamp_hlc;
pub mod hash;
pub mod json;

pub trait Module<'a> {
    fn init(&mut self, _: &mut Env<'a>, _: EnvId) {}
    fn done(&mut self, _: &mut Env<'a>, _: EnvId) {}
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a>;
}

#[cfg(not(feature = "static_module_dispatch"))]
macro_rules! for_each_module {
    ($module: ident, $vm : expr, $expr: expr) => {
        for mut $module in $vm.modules.iter_mut() {
            $expr
        }
    };
}

#[cfg(feature = "static_module_dispatch")]
macro_rules! for_each_module {
    ($module: ident, $vm : expr, $expr: expr) => {{
        {
           let ref mut $module = $vm.core;
           $expr
        }
        {
           let ref mut $module = $vm.stack;
           $expr
        }
        {
           let ref mut $module = $vm.binaries;
           $expr
        }
        {
           let ref mut $module = $vm.numbers;
           $expr
        }
        {
           let ref mut $module = $vm.storage;
           $expr
        }
        {
           let ref mut $module = $vm.hash;
           $expr
        }
        {
           let ref mut $module = $vm.hlc;
           $expr
        }
        {
           let ref mut $module = $vm.json;
           $expr
        }
    }};
}

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
    inbox: Receiver<RequestMessage>,
    sender: Sender<RequestMessage>,
    #[cfg(not(feature = "static_module_dispatch"))]
    modules: Vec<Box<Module<'a> + 'a>>,
    #[cfg(feature = "static_module_dispatch")]
    core: core::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    stack: stack::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    binaries: binaries::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    numbers: numbers::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    storage: storage::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    hash: hash::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    hlc: timestamp_hlc::Handler<'a>,
    #[cfg(feature = "static_module_dispatch")]
    json: json::Handler<'a>,
}

unsafe impl<'a> Send for VM<'a> {}

type PassResult<'a> = Result<(), Error>;

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
        let (sender, receiver) = mpsc::channel::<RequestMessage>();
        #[cfg(not(feature = "static_module_dispatch"))]
        return VM {
            inbox: receiver,
            sender: sender.clone(),
            modules: vec![Box::new(core::Handler::new(publisher)),
                          Box::new(stack::Handler::new()),
                          Box::new(binaries::Handler::new()),
                          Box::new(numbers::Handler::new()),
                          Box::new(storage::Handler::new(db_env, db)),
                          Box::new(hash::Handler::new()),
                          Box::new(timestamp_hlc::Handler::new()),
                          Box::new(json::Handler::new()),
            ],
        };
        #[cfg(feature = "static_module_dispatch")]
        return VM {
            inbox: receiver,
            sender: sender.clone(),
            core: core::Handler::new(publisher),
            stack: stack::Handler::new(),
            binaries: binaries::Handler::new(),
            numbers: numbers::Handler::new(),
            storage: storage::Handler::new(db_env, db),
            hash: hash::Handler::new(),
            hlc: timestamp_hlc::Handler::new(),
            json: json::Handler::new()
        };
    }

    pub fn sender(&self) -> Sender<RequestMessage> {
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
        let mut envs: VecDeque<(EnvId, Env<'a>, Sender<ResponseMessage>)> = VecDeque::new();

        loop {
            match envs.pop_front() {
                Some((pid, mut env, chan)) => {
                    let program = env.program[env.program.len() - 1];
                    match self.pass(&mut env, pid.clone()) {
                        Err(Error::Reschedule) => {
                            env.program.push(program);
                            envs.push_back((pid, env, chan));
                        },
                        Err(err) => {
                            for_each_module!(module, self, module.done(&mut env, pid));
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         Some(env.stack_copy()),
                                                                         Some(env.stack_size)));
                        }
                        Ok(()) => {
                            if env.program.is_empty() || (env.program.len() == 1 && env.program[0].len() == 0) {
                                for_each_module!(module, self, module.done(&mut env, pid));
                                let _ = chan.send(ResponseMessage::EnvTerminated(pid,
                                                                                 env.stack_copy(),
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
                msg.map_err(|_| mpsc::RecvError {})
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
                                    for_each_module!(module, self, module.init(&mut env, pid));
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
                            let _ = env.program.pop(); // reschedule will push it back
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
                env.push(&data[offset_by_size(data.len())..]);
            }
            if rest.len() > 0 {
                env.program.push(rest);
            }
            Ok(())
        } else if let nom::IResult::Done(rest, word) = binparser::word_or_internal_word(program) {
            if rest.len() > 0 {
                env.program.push(rest);
            }
            if word != TRY_END && !env.aborting_try.is_empty() {
                return Ok(())
            }

            try_word!(env, self.handle_try(env, word, pid));
            try_word!(env, self.handle_try_end(env, word, pid));

            for_each_module!(module, self, try_word!(env, module.handle(env, word, pid)));

            // catch-all (NB: keep it last)
            try_word!(env, self.handle_dictionary(env, word, pid));

            // if nothing worked...
            handle_error!(env, error_unknown_word!(word))
        } else {
            handle_error!(env, error_decoding!())
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
    fn handle_try(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY);
        let v = stack_pop!(env);
        env.tracking_errors += 1;
        env.program.push(TRY_END);
        env.program.push(v);
        Ok(())
    }

    #[inline]
    fn handle_try_end(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        word_is!(env, word, TRY_END);
        env.tracking_errors -= 1;
        if env.aborting_try.is_empty() {
            env.push(_EMPTY);
            Ok(())
        } else if let Some(Error::ProgramError(err)) = env.aborting_try.pop() {
            for_each_module!(module, self, module.done(env, pid));
            let slice = alloc_and_write!(err.as_slice(), env);
            env.push(slice);
            Ok(())
        } else {
            env.push(_EMPTY);
            Ok(())
        }
    }
}

pub mod compose;

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
    fn error_macro() {
        if let Error::ProgramError(err) = error_program!("Test".as_bytes(), "123".as_bytes(),b"\x01\x33") {
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
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("[\"Decoding error\" [] 5]"));
            assert_eq!(env.pop(), None);
        });

    }

    use test::Bencher;

    #[bench]
    fn ackermann(b: &mut Bencher) { // HT @5HT
        bench_eval!("['n SET 'm SET m 0 EQUAL? [n 1 UINT/ADD] \
        [n 0 EQUAL? [m 1 UINT/SUB 1 ack] [m 1 UINT/SUB m n 1 UINT/SUB ack ack] IFELSE] IFELSE] \
        'ack DEF \
        3 4 ack", b);
    }

}
