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

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK,
            ERROR_INVALID_VALUE, offset_by_size};
use timestamp;

use hlc;
use std::marker::PhantomData;
use byteorder::{BigEndian, WriteBytesExt};

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_hlc(env, word, pid));
        try_word!(env, self.handle_hlc_lc(env, word, pid));
        try_word!(env, self.handle_hlc_tick(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    pub fn handle_hlc(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC {
            let now = timestamp::hlc();
            let slice = alloc_slice!(16, env);
            let _ = now.write_bytes(&mut slice[0..]).unwrap();
            env.push(slice);
            Ok(())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_hlc_tick(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_TICK {
            let a = env.pop();

            if a.is_none() {
                return Err(error_empty_stack!());
            }

            let mut a1 = a.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() {
                return Err(error_invalid_value!(a1))
            }

            let mut t1 = t1_.unwrap();
            t1.count += 1;

            let slice = alloc_slice!(16, env);
            let _ = t1.write_bytes(&mut slice[0..]).unwrap();
            env.push(slice);

            Ok(())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_hlc_lc(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == HLC_LC {
            let a = env.pop();

            if a.is_none() {
                return Err(error_empty_stack!());
            }

            let mut a1 = a.unwrap();

            let t1_ = hlc::Timestamp::<hlc::WallT>::read_bytes(&mut a1);

            if t1_.is_err() {
                return Err(error_invalid_value!(a1))
            }

            let t1 = t1_.unwrap();

            let slice = alloc_slice!(4, env);
            let _ = (&mut slice[0..]).write_u32::<BigEndian>(t1.count);

            env.push(slice);

            Ok(())
        } else {
            Err(Error::UnknownWord)
        }
    }

}
