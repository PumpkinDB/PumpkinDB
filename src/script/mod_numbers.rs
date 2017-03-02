// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE};

use std::marker::PhantomData;

use num_bigint::{BigUint, BigInt, Sign, };
use num_traits::Signed;
use core::ops::{Add, Sub};

// Category: arithmetics
word!(UINT_ADD, (a, b => c), b"\x88UINT/ADD");
word!(UINT_SUB, (a, b => c), b"\x88UINT/SUB");
word!(INT_ADD, (a, b => c), b"\x87INT/ADD");
word!(INT_SUB, (a, b => c), b"\x87INT/SUB");

// Casting
word!(INT_TO_UINT, (a => b), b"\x89INT->UINT");
word!(UINT_TO_INT, (a => b), b"\x89UINT->INT");

// Comparison
word!(UINT_EQUALQ, (a, b => c), b"\x8BUINT/EQUAL?");
word!(UINT_GTQ, (a, b => c), b"\x88UINT/GT?");
word!(UINT_LTQ, (a, b => c), b"\x88UINT/LT?");
word!(INT_EQUALQ, (a, b => c), b"\x8AINT/EQUAL?");
word!(INT_GTQ, (a, b => c), b"\x87INT/GT?");
word!(INT_LTQ, (a, b => c), b"\x87INT/LT?");

pub fn bytes_to_bigint(bytes: &[u8]) -> Option<BigInt> {
    if bytes.len() >= 2 {
        match bytes[0] {
            0x00 => Some(Sign::Minus),
            0x01 => Some(Sign::Plus),
            _ => None
        }.and_then(|sign| Some(BigInt::from_bytes_be(sign, &bytes[1..])))
    } else {
        None
    }

}

macro_rules! bytes_to_bigint {
   ($bytes: expr) => {
       match bytes_to_bigint($bytes) {
         Some(v) => v,
         None => return Err(error_invalid_value!($bytes))
       }
   };
}

macro_rules! uint_comparison {
    ($env: expr, $word: expr, $word_const: expr, $cmp: ident) => {{
        word_is!($env, $word, $word_const);
        let b = stack_pop!($env);
        let a = stack_pop!($env);

        let a_ = BigUint::from_bytes_be(a);
        let b_ = BigUint::from_bytes_be(b);

        if a_.$cmp(&b_) {
            $env.push(STACK_TRUE);
        } else {
            $env.push(STACK_FALSE);
        }
        Ok(())
    }};
}

macro_rules! int_comparison {
    ($env: expr, $word: expr, $word_const: expr, $cmp: ident) => {{
        word_is!($env, $word, $word_const);
        let b = stack_pop!($env);
        let a = stack_pop!($env);

        let a_ = bytes_to_bigint(a);
        let b_ = bytes_to_bigint(b);

        if a_.is_none() {
            return Err(error_invalid_value!(a));
        }

        if b_.is_none() {
            return Err(error_invalid_value!(b));
        }

        if a_.unwrap().$cmp(&b_.unwrap()) {
            $env.push(STACK_TRUE);
        } else {
            $env.push(STACK_FALSE);
        }
        Ok(())
    }};
}

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_uint_add(env, word, pid));
        try_word!(env, self.handle_uint_sub(env, word, pid));
        try_word!(env, self.handle_int_add(env, word, pid));
        try_word!(env, self.handle_int_sub(env, word, pid));
        try_word!(env, self.handle_int_to_uint(env, word, pid));
        try_word!(env, self.handle_uint_to_int(env, word, pid));
        try_word!(env, self.handle_uint_equalq(env, word, pid));
        try_word!(env, self.handle_uint_gtq(env, word, pid));
        try_word!(env, self.handle_uint_ltq(env, word, pid));
        try_word!(env, self.handle_int_equalq(env, word, pid));
        try_word!(env, self.handle_int_gtq(env, word, pid));
        try_word!(env, self.handle_int_ltq(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {

    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }


    #[inline]
    fn handle_uint_add(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UINT_ADD);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_uint = BigUint::from_bytes_be(a);
        let b_uint = BigUint::from_bytes_be(b);

        let c_uint = a_uint.add(b_uint);

        let c_bytes = c_uint.to_bytes_be();

        let slice = alloc_and_write!(c_bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_add(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, INT_ADD);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_int = bytes_to_bigint(a);
        let b_int = bytes_to_bigint(b);

        if a_int == None {
            return Err(error_invalid_value!(a))
        }
        if b_int == None {
            return Err(error_invalid_value!(b))
        }

        let c_int = a_int.unwrap().add(b_int.unwrap());

        let mut bytes = if c_int.is_negative() {
            vec![0x00]
        } else {
            vec![0x01]
        };
        let (_, c_bytes) = c_int.to_bytes_be();
        bytes.extend_from_slice(&c_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_sub(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, INT_SUB);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_int = bytes_to_bigint(a);
        let b_int = bytes_to_bigint(b);

        if a_int == None {
            return Err(error_invalid_value!(a))
        }
        if b_int == None {
            return Err(error_invalid_value!(b))
        }

        let c_int = b_int.unwrap().sub(a_int.unwrap());

        let mut bytes = if c_int.is_negative() {
            vec![0x00]
        } else {
            vec![0x01]
        };
        let (_, c_bytes) = c_int.to_bytes_be();
        bytes.extend_from_slice(&c_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_to_uint(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId)
                          -> PassResult<'a> {
        word_is!(env, word, INT_TO_UINT);
        let a = stack_pop!(env);
        let a_int = bytes_to_bigint(a);

        if a_int == None {
            return Err(error_invalid_value!(a))
        }

        match a_int.unwrap().to_biguint() {
            Some(a_uint) => {
                let a_bytes = a_uint.to_bytes_be();
                let slice = alloc_and_write!(a_bytes.as_slice(), env);
                env.push(slice);
                Ok(())
            },
            None => {
                Err(error_invalid_value!(a))
            }
        }
    }

    fn handle_uint_to_int(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId)
                          -> PassResult<'a> {
        word_is!(env, word, UINT_TO_INT);
        let a = stack_pop!(env);
        let a_uint = BigUint::from_bytes_be(a);

        let mut bytes = vec![0x01];
        let a_bytes = a_uint.to_bytes_be();
        bytes.extend_from_slice(&a_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);

        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_uint_sub(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, UINT_SUB);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_uint = BigUint::from_bytes_be(a);
        let b_uint = BigUint::from_bytes_be(b);

        if a_uint > b_uint {
            return Err(error_invalid_value!(a));
        }

        let c_uint = b_uint.sub(a_uint);

        let c_bytes = c_uint.to_bytes_be();
        let slice = alloc_and_write!(c_bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_uint_equalq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, word, UINT_EQUALQ, eq)
    }

    #[inline]
    fn handle_uint_gtq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, word, UINT_GTQ, gt)
    }

    #[inline]
    fn handle_uint_ltq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, word, UINT_LTQ, lt)
    }

    #[inline]
    fn handle_int_equalq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, word, INT_EQUALQ, eq)
    }

    #[inline]
    fn handle_int_gtq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, word, INT_GTQ, gt)
    }

    #[inline]
    fn handle_int_ltq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, word, INT_LTQ, lt)
    }

}