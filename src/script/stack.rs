// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, binparser};

use std::marker::PhantomData;

use nom;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

word!(DROP, (a => ), b"\x84DROP");
word!(DUP, (a => a, a), b"\x83DUP");
word!(SWAP, (a, b => b, a), b"\x84SWAP");
word!(TWOSWAP, (a, b, c, d => c, d, a, b), b"\x852SWAP");
word!(ROT, (a, b, c  => b, c, a), b"\x83ROT");
word!(TWOROT, (a, b, c, d, e, f  => c, d, e, f, a, b), b"\x842ROT");
word!(OVER, (a, b => a, b, a), b"\x84OVER");
word!(TWOOVER, (a, b, c, d => a, b, c, d, a, b), b"\x852OVER");
word!(DEPTH, b"\x85DEPTH");
word!(UNWRAP, b"\x86UNWRAP");
word!(WRAP, b"\x84WRAP");

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_drop(env, word, pid));
        try_word!(env, self.handle_dup(env, word, pid));
        try_word!(env, self.handle_swap(env, word, pid));
        try_word!(env, self.handle_2swap(env, word, pid));
        try_word!(env, self.handle_rot(env, word, pid));
        try_word!(env, self.handle_2rot(env, word, pid));
        try_word!(env, self.handle_over(env, word, pid));
        try_word!(env, self.handle_2over(env, word, pid));
        try_word!(env, self.handle_depth(env, word, pid));
        try_word!(env, self.handle_wrap(env, word, pid));
        try_word!(env, self.handle_unwrap(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {

    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    fn handle_dup(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DUP);
        let v = stack_pop!(env);

        env.push(v);
        env.push(v);
        Ok(())
    }

    #[inline]
    fn handle_swap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, SWAP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_2swap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TWOSWAP);
        let a = stack_pop!(env);
        let b = stack_pop!(env);
        let c = stack_pop!(env);
        let d = stack_pop!(env);

        env.push(b);
        env.push(a);

        env.push(d);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_over(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, OVER);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_2over(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TWOOVER);
        let d = stack_pop!(env);
        let c = stack_pop!(env);
        let b = stack_pop!(env);
        let a = stack_pop!(env);

        env.push(a);
        env.push(b);
        env.push(c);
        env.push(d);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_rot(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, ROT);
        let a = stack_pop!(env);
        let b = stack_pop!(env);
        let c = stack_pop!(env);

        env.push(b);
        env.push(a);
        env.push(c);

        Ok(())
    }

    #[inline]
    fn handle_2rot(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, TWOROT);
        let f = stack_pop!(env);
        let e = stack_pop!(env);
        let d = stack_pop!(env);
        let c = stack_pop!(env);
        let b = stack_pop!(env);
        let a = stack_pop!(env);

        env.push(c);
        env.push(d);
        env.push(e);
        env.push(f);
        env.push(a);
        env.push(b);

        Ok(())
    }

    #[inline]
    fn handle_drop(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DROP);
        let _ = stack_pop!(env);

        Ok(())
    }

    #[inline]
    fn handle_depth(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, DEPTH);
        let bytes = BigUint::from(env.stack_size).to_bytes_be();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_wrap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, WRAP);
        let n = stack_pop!(env);

        let mut n_int = BigUint::from_bytes_be(n).to_u64().unwrap() as usize;

        let mut vec = Vec::new();

        while n_int > 0 {
            let item = stack_pop!(env);
            vec.insert(0, item);
            n_int -= 1;
        }

        let size = vec.clone().into_iter()
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
    fn handle_unwrap(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UNWRAP);
        let mut current = stack_pop!(env);
        while current.len() > 0 {
            match binparser::data(current) {
                nom::IResult::Done(rest, val) => {
                    env.push(&val[offset_by_size(val.len())..]);
                    current = rest
                },
                _ => {
                    return Err(error_invalid_value!(current))
                }
            }
        }
        Ok(())
    }

}