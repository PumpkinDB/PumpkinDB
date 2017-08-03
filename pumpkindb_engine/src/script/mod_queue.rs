// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pumpkinscript::{offset_by_size};
use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_NO_VALUE,
            TryInstruction, STACK_TRUE, STACK_FALSE};

use std::marker::PhantomData;

instruction!(TO_BQ, b"\x82>Q");
instruction!(FROM_BQ, b"\x82Q>");
instruction!(TO_FQ, b"\x82<Q");
instruction!(FROM_FQ, b"\x82Q<");
instruction!(QQ, b"\x82Q?");

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_to_bq(env, instruction, pid)
        .if_unhandled_try(|| self.handle_from_bq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_to_fq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_from_fq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_qq(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    fn handle_to_bq(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TO_BQ);
        let val = env.pop().ok_or_else(|| error_empty_stack!())?;
        env.queue_back_push(val);
        Ok(())
    }

    #[inline]
    fn handle_from_bq(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, FROM_BQ);
        match env.queue_back_pop() {
            Some(value) => {
                env.push(value);
                Ok(())
            }
            None => Err(error_no_value!())
        }
    }

    #[inline]
    fn handle_to_fq(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TO_FQ);
        let val = env.pop().ok_or_else(|| error_empty_stack!())?;
        env.queue_front_push(val);
        Ok(())
    }

    #[inline]
    fn handle_from_fq(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, FROM_FQ);
        match env.queue_front_pop() {
            Some(value) => {
                env.push(value);
                Ok(())
            }
            None => Err(error_no_value!())
        }
    }

    #[inline]
    fn handle_qq(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, QQ);
        if env.queue_empty() {
            env.push(STACK_FALSE);
        } else {
            env.push(STACK_TRUE);
        }
        Ok(())
    }

}
