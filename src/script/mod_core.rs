// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE, binparser, textparser, parse_bin};
use super::super::pubsub;

use std::marker::PhantomData;

use std::collections::BTreeMap;

use nom;
use num_bigint::BigUint;
use num_traits::{Zero, One};
use core::ops::Sub;

// Category: Control flow
#[cfg(feature = "scoped_dictionary")]
word!(EVAL_SCOPED, b"\x8BEVAL/SCOPED");
#[cfg(feature = "scoped_dictionary")]
word!(SCOPE_END, b"\x80\x8BEVAL/SCOPED"); // internal word
word!(DOWHILE, b"\x87DOWHILE");
word!(TIMES, b"\x85TIMES");
word!(EVAL, b"\x84EVAL");
word!(EVAL_VALIDP, b"\x8BEVAL/VALID?");
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

pub struct Handler<'a> {
    publisher: pubsub::PublisherAccessor<Vec<u8>>,
    phantom: PhantomData<&'a ()>
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_builtins(env, word, pid));
        try_word!(env, self.handle_dowhile(env, word, pid));
        try_word!(env, self.handle_times(env, word, pid));
        try_word!(env, self.handle_scope_end(env, word, pid));
        try_word!(env, self.handle_eval(env, word, pid));
        try_word!(env, self.handle_eval_validp(env, word, pid));
        try_word!(env, self.handle_eval_scoped(env, word, pid));
        try_word!(env, self.handle_set(env, word, pid));
        try_word!(env, self.handle_def(env, word, pid));
        try_word!(env, self.handle_not(env, word, pid));
        try_word!(env, self.handle_and(env, word, pid));
        try_word!(env, self.handle_or(env, word, pid));
        try_word!(env, self.handle_ifelse(env, word, pid));
        try_word!(env, self.handle_send(env, word, pid));
        try_word!(env, self.handle_featurep(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {

    pub fn new(publisher: pubsub::PublisherAccessor<Vec<u8>>) -> Self {
        Handler { publisher: publisher, phantom: PhantomData }
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
        if parse_bin(a).is_ok() {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
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

    #[inline]
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

    use script::{Env, Scheduler, Error, RequestMessage, ResponseMessage, EnvId, parse, offset_by_size};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::fs;
    use std::thread;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use super::binparser;
    use pubsub;
    use storage;
    use timestamp;

    const _EMPTY: &'static [u8] = b"";

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

    use test::Bencher;

    #[bench]
    fn times(b: &mut Bencher) {
        bench_eval!("[1 DROP] 1000 TIMES", b);
    }

}
