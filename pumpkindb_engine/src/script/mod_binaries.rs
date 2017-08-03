// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE, TryInstruction};

use std::marker::PhantomData;

use num_bigint::BigUint;
use num_traits::ToPrimitive;

instruction!(EQUALQ, (a, b => c), b"\x86EQUAL?");
instruction!(LTQ, (a, b => c), b"\x83LT?");
instruction!(GTQ, (a, b => c), b"\x83GT?");
instruction!(LENGTH, (a => b), b"\x86LENGTH");
instruction!(CONCAT, (a, b => c), b"\x86CONCAT");
instruction!(SLICE, (a, b, c => d), b"\x85SLICE");
instruction!(PAD, (a, b, c => d), b"\x83PAD");


pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

builtins!("mod_binaries.psc");

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_builtins(env, instruction, pid)
        .if_unhandled_try(|| self.handle_ltp(env, instruction, pid))
        .if_unhandled_try(|| self.handle_gtp(env, instruction, pid))
        .if_unhandled_try(|| self.handle_equal(env, instruction, pid))
        .if_unhandled_try(|| self.handle_concat(env, instruction, pid))
        .if_unhandled_try(|| self.handle_slice(env, instruction, pid))
        .if_unhandled_try(|| self.handle_pad(env, instruction, pid))
        .if_unhandled_try(|| self.handle_length(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    handle_builtins!();

    #[inline]
    fn handle_equal(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, EQUALQ);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        if a == b {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_ltp(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, LTQ);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        if b < a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_gtp(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, GTQ);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        if b > a {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_concat(&mut self,
                     env: &mut Env<'a>,
                     instruction: &'a [u8],
                     _: EnvId)
                     -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CONCAT);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        let slice = alloc_slice!(a.len() + b.len(), env);

        slice[0..b.len()].copy_from_slice(b);
        slice[b.len()..b.len() + a.len()].copy_from_slice(a);

        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_slice(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, SLICE);
        let end = env.pop().ok_or_else(|| error_empty_stack!())?;
        let start = env.pop().ok_or_else(|| error_empty_stack!())?;
        let slice = env.pop().ok_or_else(|| error_empty_stack!())?;

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
    fn handle_pad(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, PAD);
        let byte = env.pop().ok_or_else(|| error_empty_stack!())?;
        let size = env.pop().ok_or_else(|| error_empty_stack!())?;
        let value = env.pop().ok_or_else(|| error_empty_stack!())?;

        if byte.len() != 1 {
            return Err(error_invalid_value!(byte));
        }

        let size_int = BigUint::from_bytes_be(size).to_u64().unwrap() as usize;

        if size_int > 1024 {
            return Err(error_invalid_value!(size));
        }

        if size_int < value.len() {
            return Err(error_invalid_value!(size));
        }

        let slice = alloc_slice!(size_int, env);

        for i in 0..size_int - value.len() {
            slice[i] = byte[0];
        }
        slice[size_int - value.len()..].copy_from_slice(value);

        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_length(&mut self,
                     env: &mut Env<'a>,
                     instruction: &'a [u8],
                     _: EnvId)
                     -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, LENGTH);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let len = BigUint::from(a.len() as u64);
        let len_bytes = len.to_bytes_be();

        let slice = alloc_and_write!(len_bytes.as_slice(), env);

        env.push(slice);

        Ok(())
    }
}
