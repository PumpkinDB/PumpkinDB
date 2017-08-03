// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pumpkinscript::{offset_by_size, binparser};
use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK,
            ERROR_INVALID_VALUE, TryInstruction};

use std::marker::PhantomData;

use pumpkinscript;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

instruction!(THREEDROP, (a, b, c => ), b"\x853DROP");
instruction!(THREEDUP, (a, b, c => a, b, c), b"\x843DUP");
instruction!(DROP, (a => ), b"\x84DROP");
instruction!(DUP, (a => a, a), b"\x83DUP");
instruction!(SWAP, (a, b => b, a), b"\x84SWAP");
instruction!(TWOSWAP, (a, b, c, d => c, d, a, b), b"\x852SWAP");
instruction!(ROT, (a, b, c  => b, c, a), b"\x83ROT");
instruction!(TWOROT, (a, b, c, d, e, f  => c, d, e, f, a, b), b"\x842ROT");
instruction!(OVER, (a, b => a, b, a), b"\x84OVER");
instruction!(TWOOVER, (a, b, c, d => a, b, c, d, a, b), b"\x852OVER");
instruction!(DEPTH, b"\x85DEPTH");
instruction!(UNWRAP, b"\x86UNWRAP");
instruction!(WRAP, b"\x84WRAP");
instruction!(PUSH, b"\x81<");
instruction!(POP, b"\x81>");

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

builtins!("mod_stack.psc");

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_builtins(env, instruction, pid)
        .if_unhandled_try(|| self.handle_drop(env, instruction, pid))
        .if_unhandled_try(|| self.handle_dup(env, instruction, pid))
        .if_unhandled_try(|| self.handle_3drop(env, instruction, pid))
        .if_unhandled_try(|| self.handle_3dup(env, instruction, pid))
        .if_unhandled_try(|| self.handle_swap(env, instruction, pid))
        .if_unhandled_try(|| self.handle_2swap(env, instruction, pid))
        .if_unhandled_try(|| self.handle_rot(env, instruction, pid))
        .if_unhandled_try(|| self.handle_2rot(env, instruction, pid))
        .if_unhandled_try(|| self.handle_over(env, instruction, pid))
        .if_unhandled_try(|| self.handle_2over(env, instruction, pid))
        .if_unhandled_try(|| self.handle_depth(env, instruction, pid))
        .if_unhandled_try(|| self.handle_wrap(env, instruction, pid))
        .if_unhandled_try(|| self.handle_unwrap(env, instruction, pid))
        .if_unhandled_try(|| self.handle_push(env, instruction, pid))
        .if_unhandled_try(|| self.handle_pop(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    handle_builtins!();

    #[inline]
    fn handle_dup(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, DUP);
        let v = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(v);
        env.push(v);
        Ok(())
    }


    #[inline]
    fn handle_3dup(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, THREEDUP);
        let c = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(a);
        env.push(b);
        env.push(c);

        env.push(a);
        env.push(b);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_swap(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, SWAP);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_2swap(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TWOSWAP);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let c = env.pop().ok_or_else(|| error_empty_stack!())?;
        let d = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(b);
        env.push(a);

        env.push(d);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_over(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, OVER);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(b);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_2over(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TWOOVER);
        let d = env.pop().ok_or_else(|| error_empty_stack!())?;
        let c = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(a);
        env.push(b);
        env.push(c);
        env.push(d);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_rot(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, ROT);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let c = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(b);
        env.push(a);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_2rot(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TWOROT);
        let f = env.pop().ok_or_else(|| error_empty_stack!())?;
        let e = env.pop().ok_or_else(|| error_empty_stack!())?;
        let d = env.pop().ok_or_else(|| error_empty_stack!())?;
        let c = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        env.push(c);
        env.push(d);
        env.push(e);
        env.push(f);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_drop(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, DROP);
        let _ = env.pop().ok_or_else(|| error_empty_stack!())?;

        Ok(())
    }

    #[inline]
    fn handle_3drop(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, THREEDROP);
        let _ = env.pop().ok_or_else(|| error_empty_stack!())?;
        let _ = env.pop().ok_or_else(|| error_empty_stack!())?;
        let _ = env.pop().ok_or_else(|| error_empty_stack!())?;

        Ok(())
    }

    #[inline]
    fn handle_depth(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, DEPTH);
        let stack_size = env.stack().len();
        let bytes = BigUint::from(stack_size).to_bytes_be();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_wrap(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, WRAP);
        let n = env.pop().ok_or_else(|| error_empty_stack!())?;

        let mut n_int = BigUint::from_bytes_be(n).to_u64().unwrap() as usize;

        let mut vec = Vec::new();

        while n_int > 0 {
            let item = env.pop().ok_or_else(|| error_empty_stack!())?;
            vec.insert(0, item);
            n_int -= 1;
        }

        let size = vec.clone()
            .into_iter()
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
    fn handle_unwrap(&mut self,
                     env: &mut Env<'a>,
                     instruction: &'a [u8],
                     _: EnvId)
                     -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UNWRAP);
        let mut current = env.pop().ok_or_else(|| error_empty_stack!())?;
        while current.len() > 0 {
            match binparser::data(current) {
                pumpkinscript::ParseResult::Done(rest, val) => {
                    env.push(&val[offset_by_size(val.len())..]);
                    current = rest
                }
                _ => return Err(error_invalid_value!(current)),
            }
        }
        Ok(())
    }

    #[inline]
    fn handle_push(&mut self,
                     env: &mut Env<'a>,
                     instruction: &'a [u8],
                     _: EnvId)
                     -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, PUSH);
        env.push_stack();
        Ok(())
    }

    #[inline]
    fn handle_pop(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, POP);
        if !env.pop_stack() {
            Err(error_empty_stack!())
        } else {
            Ok(())
        }
    }

}
