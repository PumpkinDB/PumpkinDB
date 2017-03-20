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
instruction!(HLC, b"\x83HLC");
instruction!(HLC_LC, b"\x86HLC/LC");
instruction!(HLC_TICK, b"\x88HLC/TICK");

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK,
            ERROR_INVALID_VALUE, offset_by_size};
use timestamp;

use hlc;
use std::marker::PhantomData;
use byteorder::{BigEndian, WriteBytesExt};
use std::sync::Arc;

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
    timestamp: Arc<timestamp::Timestamp>,
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_instruction!(env, self.handle_hlc(env, instruction, pid));
        try_instruction!(env, self.handle_hlc_lc(env, instruction, pid));
        try_instruction!(env, self.handle_hlc_tick(env, instruction, pid));
        Err(Error::UnknownInstruction)
    }
}

impl<'a> Handler<'a> {
    pub fn new(timestamp_state: Arc<timestamp::Timestamp>) -> Self {
        Handler {
            phantom: PhantomData,
            timestamp: timestamp_state
        }
    }

    #[inline]
    pub fn handle_hlc(&self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        if instruction == HLC {
            let now = self.timestamp.hlc();
            let slice = alloc_slice!(16, env);
            let _ = now.write_bytes(&mut slice[0..]).unwrap();
            env.push(slice);
            Ok(())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_hlc_tick(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        if instruction == HLC_TICK {
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
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_hlc_lc(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        if instruction == HLC_LC {
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
            Err(Error::UnknownInstruction)
        }
    }

}
