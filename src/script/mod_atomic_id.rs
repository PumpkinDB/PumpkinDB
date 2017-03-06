// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Atomic Counter Value: ACV
//!
//! This module handles acces to a global usize count for increasing count needs
//!
//!


//Word Decleration 
word!(ACV, b"\x83ACV");

use std::marker::PhantomData;
use super::{Module, PassResult, Error, Env, EnvId};
use logicalstamp;
use byteorder::{ByteOrder, BigEndian};

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_acv(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    pub fn handle_acv(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        if word == ACV {
            let c = logicalstamp::acv_count();

            if cfg!(target_pointer_width = "32"){
                let mut buf = [0; 4];
                let val = c as u32;
                BigEndian::write_u32(&mut buf, val);
                let slice = alloc_and_write!(&buf, env);
                env.push(slice);
            }

            if cfg!(target_pointer_width = "64") {
                let mut buf = [0; 8];
                let val = c as u64;
                BigEndian::write_u64(&mut buf, val);
                let slice = alloc_and_write!(&buf, env);
                env.push(slice);
            }
            //env.push(slice);
            Ok(())
        }else {
            //Does not match word 
            Err(Error::UnknownWord)
        }
    }

}

