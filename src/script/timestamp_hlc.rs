// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Timestamp: HLC
//!
//! This module handles issuance, management and comparison of Hybrid
//! Logical Clock timestamps (https://www.cse.buffalo.edu/tech-reports/2014-04.pdf)
//!
word!(HLC, b"\x83HLC");
word!(HLC_LC, b"\x86HLC/LC");
word!(HLC_TICK, b"\x88HLC/TICK");
word!(HLC_LTP, b"\x87HLC/LT?");
word!(HLC_GTP, b"\x87HLC/GT?");

use super::{Env, EnvId, PassResult, Error, STACK_TRUE, STACK_FALSE};
use timestamp;

use hlc;
use std::marker::PhantomData;
use byteorder::{BigEndian, WriteBytesExt};


pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    pub fn handle_hlc(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC {
            let now = timestamp::hlc();
            let slice0 = env.alloc(16);
            if slice0.is_err() {
                return Err((env, slice0.unwrap_err()));
            }
            let mut slice = slice0.unwrap();
            let _ = now.write_bytes(&mut slice[0..]).unwrap();
            env.push(slice);
            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_hlc_ltp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_LTP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let mut a1 = a.unwrap();
            let mut b1 = b.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut b1);
            let t2_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() || t2_.is_err() {
                return Err((env, Error::InvalidValue))
            }

            let t1 = t1_.unwrap();
            let t2 = t2_.unwrap();

            if t1 < t2 {
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
    pub fn handle_hlc_gtp(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_GTP {
            let a = env.pop();
            let b = env.pop();

            if a.is_none() || b.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let mut a1 = a.unwrap();
            let mut b1 = b.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut b1);
            let t2_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() || t2_.is_err() {
                return Err((env, Error::InvalidValue))
            }

            let t1 = t1_.unwrap();
            let t2 = t2_.unwrap();

            if t1 > t2 {
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
    pub fn handle_hlc_tick(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_TICK {
            let a = env.pop();

            if a.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let mut a1 = a.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() {
                return Err((env, Error::InvalidValue))
            }

            let mut t1 = t1_.unwrap();
            t1.count += 1;

            let slice0 = env.alloc(16);
            if slice0.is_err() {
                return Err((env, slice0.unwrap_err()));
            }
            let slice = slice0.unwrap();
            let _ = t1.write_bytes(&mut slice[0..]).unwrap();
            env.push(slice);

            Ok((env, None))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_hlc_lc(&mut self, mut env: Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_LC {
            let a = env.pop();

            if a.is_none() {
                return Err((env, Error::EmptyStack));
            }

            let mut a1 = a.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() {
                return Err((env, Error::InvalidValue))
            }

            let t1 = t1_.unwrap();

            let slice0 = env.alloc(4);
            if slice0.is_err() {
                return Err((env, slice0.unwrap_err()));
            }
            let slice = slice0.unwrap();
            let _ = (&mut slice[0..]).write_u32::<BigEndian>(t1.count);

            env.push(slice);

            Ok((env, None))
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
    use script::binparser;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn hlc() {
        eval!("HLC HLC HLC/LT? HLC HLC SWAP HLC/GT?",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
              });

        eval!("1 2 HLC/LT?",
              env,
              result,
              {
                  assert!(result.is_err());
              });

        eval!("1 2 HLC/GT?",
              env,
              result,
              {
                  assert!(result.is_err());
              });

        eval!("HLC DUP HLC/TICK HLC/LT?",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
              });

        eval!("1 HLC/TICK",
              env,
              result,
              {
                  assert!(result.is_err());
              });


        eval!("HLC DUP HLC/LC SWAP HLC/TICK HLC/LC",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), vec![0, 0, 0, 1]);
                  assert_eq!(Vec::from(env.pop().unwrap()), vec![0, 0, 0, 0]);
              });

        eval!("1 HLC/LC",
              env,
              result,
              {
                  assert!(result.is_err());
              });

    }

}