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

pub mod envheap;
pub mod dispatcher;
pub use self::dispatcher::Dispatcher;

use super::messaging;

const _EMPTY: &'static [u8] = b"";
  
/// `instruction!` macro is used to define a built-in instruction, its signature (if applicable)
/// and representation
macro_rules! instruction {
    ($name : ident,
    ($($input : ident),* => $($output : ident),*),
    $ident : expr) =>
    (
     instruction!($name, $ident);
    );
    ($name : ident,
    $ident : expr) =>
    (
     const $name : &'static[u8] = $ident;
    )
}

instruction!(TRY, b"\x83TRY");
instruction!(TRY_END, b"\x80\x83TRY"); // internal instruction


use std::str;

// To add instructions that don't belong to a core set,
// add a module with a handler, and reference it in the Scheduler's pass

pub type Program = Vec<u8>;

/// `Error` represents an enumeration of possible `Executor` errors.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// Instruction is unknown
    UnknownInstruction,
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
    Superfluous(Vec<u8>),
}

pub mod env;
pub use self::env::Env;

use pumpkinscript;

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

/// Communication messages used to talk with the [Scheduler](struct.Scheduler.html) thread.
pub enum RequestMessage {
    /// Requests scheduling a new environment with a given
    /// id and a program.
    ScheduleEnv(EnvId, Vec<u8>, Sender<ResponseMessage>,
                Box<messaging::PublishedMessageCallback + Send>),
    /// Requests Scheduler shutdown
    Shutdown,
}

/// Messages received from the [Scheduler](struct.Scheduler.html) thread.
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

use storage;
use timestamp;

#[cfg(feature="mod_core")]
pub mod mod_core;
#[cfg(feature="mod_stack")]
pub mod mod_stack;
#[cfg(feature="mod_numbers")]
pub mod mod_numbers;
#[cfg(feature="mod_binaries")]
pub mod mod_binaries;
#[cfg(feature="mod_storage")]
pub mod mod_storage;
#[cfg(feature="mod_hlc")]
pub mod mod_hlc;
#[cfg(feature="mod_hash")]
pub mod mod_hash;
#[cfg(feature="mod_json")]
pub mod mod_json;
#[cfg(feature="mod_msg")]
pub mod mod_msg;
#[cfg(feature="mod_uuid")]
pub mod mod_uuid;
#[cfg(feature="mod_string")]
pub mod mod_string;

/// Scheduler is a PumpkinScript scheduler and interpreter. This is the
/// most central part of this module.
///
/// # Example
///
/// ```norun
/// let mut scheduler = Scheduler::new(dispatcher, receiver);
///
/// let sender = scheduler.sender();
/// let handle = thread::spawn(move || {
///     scheduler.run();
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

use std::marker::PhantomData;

pub struct Scheduler<'a, T : Dispatcher<'a>> {
    inbox: Receiver<RequestMessage>,
    dispatcher: T,
    phantom: PhantomData<&'a ()>,
}

unsafe impl<'a, T : Dispatcher<'a>> Send for Scheduler<'a, T> {}

type PassResult<'a> = Result<(), Error>;

const STACK_TRUE: &'static [u8] = b"\x01";
const STACK_FALSE: &'static [u8] = b"\x00";

const ERROR_UNKNOWN_INSTRUCTION: &'static [u8] = b"\x01\x02";
const ERROR_INVALID_VALUE: &'static [u8] = b"\x01\x03";
const ERROR_EMPTY_STACK: &'static [u8] = b"\x01\x04";
const ERROR_DECODING: &'static [u8] = b"\x01\x05";
const ERROR_DUPLICATE_KEY: &'static [u8] = b"\x01\x06";
const ERROR_UNKNOWN_KEY: &'static [u8] = b"\x01\x07";
const ERROR_NO_TX: &'static [u8] = b"\x01\x08";
const ERROR_DATABASE: &'static [u8] = b"\x01\x09";
const ERROR_NO_VALUE: &'static [u8] = b"\x01\x0A";

use std::sync::Arc;

use pumpkinscript::{binparser};

impl<'a, T: Dispatcher<'a>> Scheduler<'a, T> {
    /// Creates an instance of Scheduler with three communication channels:
    pub fn new(dispatcher: T,
               receiver: Receiver<RequestMessage>)
               -> Self {
        Scheduler::<'a, T> {
            inbox: receiver,
            dispatcher: dispatcher,
            phantom: PhantomData,
        }
    }

    pub fn create_sender() -> (Sender<RequestMessage>, Receiver<RequestMessage>) {
        mpsc::channel::<RequestMessage>()
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
                        }
                        Err(err) => {
                            self.dispatcher.done(&mut env, pid);
                            let stack_size = env.stack_size;
                            let _ = chan.send(ResponseMessage::EnvFailed(pid,
                                                                         err,
                                                                         Some(env.stack_copy()),
                                                                         Some(stack_size)));
                        }
                        Ok(()) => {
                            if env.program.is_empty() ||
                                (env.program.len() == 1 && env.program[0].len() == 0) {
                                self.dispatcher.done(&mut env, pid);
                                let stack_size = env.stack_size;
                                let _ = chan.send(ResponseMessage::EnvTerminated(pid,
                                                                                 env.stack_copy(),
                                                                                 stack_size));
                            } else {
                                envs.push_back((pid, env, chan));
                            }
                        }
                    };
                }
                None => (),
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
                Ok(RequestMessage::ScheduleEnv(pid, program, chan, cb)) => {
                    match Env::new() {
                        Ok(mut env) => {
                            env.set_published_message_callback(cb);
                            match env.alloc(program.len()) {
                                Ok(slice) => {
                                    slice.copy_from_slice(program.as_slice());
                                    env.program.push(slice);
                                    self.dispatcher.init(&mut env, pid);
                                    envs.push_back((pid, env, chan));
                                }
                                Err(err) => {
                                    let _ =
                                        chan.send(ResponseMessage::EnvFailed(pid, err, None, None));
                                }
                            }
                        }
                        Err(err) => {
                            let _ = chan.send(ResponseMessage::EnvFailed(pid, err, None, None));
                        }
                    }
                }
            }
        }
    }

    #[allow(unused_mut)]
    fn pass(&mut self, env: &mut Env<'a>, pid: EnvId) -> PassResult<'a> {
        if env.program.len() == 0 {
            return Ok(());
        }
        let program = env.program.pop().unwrap();
        if program.len() == 0 {
            return Ok(());
        }
        if let pumpkinscript::ParseResult::Done(rest, data) = binparser::data(program) {
            if env.aborting_try.is_empty() {
                env.push(&data[offset_by_size(data.len())..]);
            }
            if rest.len() > 0 {
                env.program.push(rest);
            }
            Ok(())
        } else if let pumpkinscript::ParseResult::Done(rest, instruction) =
        binparser::instruction_or_internal_instruction(program) {
            if rest.len() > 0 {
                env.program.push(rest);
            }
            if instruction != TRY_END && !env.aborting_try.is_empty() {
                return Ok(());
            }

            try_instruction!(env, self.handle(env, instruction, pid));

            // if nothing worked...
            handle_error!(env, error_unknown_instruction!(instruction))
        } else {
            handle_error!(env, error_decoding!())
        }
    }


    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_dictionary(&mut self,
                         env: &mut Env<'a>,
                         instruction: &'a [u8],
                         _: EnvId)
                         -> PassResult<'a> {
        if env.dictionary.contains_key(instruction) {
            {
                let def = env.dictionary.get(instruction).unwrap();
                env.program.push(def);
            }
            Ok(())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_dictionary(&mut self,
                         env: &mut Env<'a>,
                         instruction: &'a [u8],
                         _: EnvId)
                         -> PassResult<'a> {
        let mut found = false;

        for i in (0..env.dictionary.len()).rev() {
            let ref dict = env.dictionary[i];
            if let Some(def) = dict.get(instruction) {
                env.program.push(def);
                found = true;
                break;
            }
        }

        if found {
            Ok(())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    fn handle_try(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        instruction_is!(instruction, TRY);
        let v = stack_pop!(env);
        env.tracking_errors += 1;
        env.program.push(TRY_END);
        env.program.push(v);
        Ok(())
    }

    #[inline]
    fn handle_try_end(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      pid: EnvId)
                      -> PassResult<'a> {
        instruction_is!(instruction, TRY_END);
        env.tracking_errors -= 1;
        if env.aborting_try.is_empty() {
            env.push(_EMPTY);
            Ok(())
        } else if let Some(Error::ProgramError(err)) = env.aborting_try.pop() {
            self.dispatcher.done(env, pid);
            let slice = alloc_and_write!(err.as_slice(), env);
            env.push(slice);
            Ok(())
        } else {
            env.push(_EMPTY);
            Ok(())
        }
    }
}

impl<'a, T: Dispatcher<'a>> Dispatcher<'a> for Scheduler<'a, T> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_instruction!(env, self.handle_try(env, instruction, pid));
        try_instruction!(env, self.handle_try_end(env, instruction, pid));

        try_instruction!(env, self.dispatcher.handle(env, instruction, pid));

        // catch-all (NB: keep it last)
        try_instruction!(env, self.handle_dictionary(env, instruction, pid));
        Err(Error::UnknownInstruction)
    }
}


#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use pumpkinscript::{parse, offset_by_size};
    use messaging;
    use nvmem::{MmapedFile, MmapedRegion, NonVolatileMemory};
    use script::{Env, Scheduler, Error, RequestMessage, ResponseMessage, EnvId, dispatcher};
    use std::sync::mpsc;
    use std::sync::Arc;
    use timestamp;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use super::binparser;
    use storage;
    use rand::Rng;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn error_macro() {
        if let Error::ProgramError(err) =
            error_program!("Test".as_bytes(), "123".as_bytes(), b"\x01\x33") {
            assert_eq!(err, parsed_data!("[\"Test\" [\"123\"] 0x33]"));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn unknown_instruction() {
        eval!("NOTANINSTRUCTION", env, result, {
            assert_error!(result,
                          "[\"Unknown instruction: NOTANINSTRUCTION\" ['NOTANINSTRUCTION] 2]");
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
            assert_eq!(Vec::from(env.pop().unwrap()),
                       parsed_data!("[\"Empty stack\" [] 4]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[NOTANINSTRUCTION] TRY", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()),
                       parsed_data!("[\"Unknown instruction: NOTANINSTRUCTION\" \
                                     ['NOTANINSTRUCTION] 2]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[[DUP] TRY 0x20 NOT] TRY", env, result, {
            assert!(!result.is_err());
            assert_eq!(Vec::from(env.pop().unwrap()),
                       parsed_data!("[\"Invalid value\" [0x20] 3]"));
            assert_eq!(Vec::from(env.pop().unwrap()),
                       parsed_data!("[\"Empty stack\" [] 4]"));
            assert_eq!(env.pop(), None);
        });

        eval!("[1 DUP] TRY STACK DROP DUP", env, result, {
            assert!(result.is_err());
        });

        eval!("[DUP] TRY STACK DROP DUP", env, result, {
            assert!(result.is_err());
        });

        eval!("1 TRY", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()),
                       parsed_data!("[\"Decoding error\" [] 5]"));
            assert_eq!(env.pop(), None);
        });

    }

    use test::Bencher;

    #[bench]
    fn ackermann(b: &mut Bencher) {
        // HT @5HT
        bench_eval!("['n SET 'm SET m 0 EQUAL? [n 1 UINT/ADD] \
        [n 0 EQUAL? [m 1 UINT/SUB 1 ack] [m 1 UINT/SUB m n 1 UINT/SUB ack ack] IFELSE] IFELSE] \
        'ack DEF \
        3 4 ack",
                    b);
    }

    #[bench]
    fn ackermann_stack(b: &mut Bencher) {
        // HT @5HT
        bench_eval!("[OVER 0 EQUAL? [1 UINT/ADD NIP] \
        [DUP 0 EQUAL? [DROP 1 UINT/SUB 1 ack] [OVER 1 UINT/SUB -ROT 1 UINT/SUB ack ack] IFELSE] IFELSE] \
        'ack DEF \
        3 4 ack",
                    b);
    }

}
