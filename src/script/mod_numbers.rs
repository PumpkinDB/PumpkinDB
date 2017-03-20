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
instruction!(UINT_ADD, (a, b => c), b"\x88UINT/ADD");
instruction!(UINT_SUB, (a, b => c), b"\x88UINT/SUB");
instruction!(INT_ADD, (a, b => c), b"\x87INT/ADD");
instruction!(INT_SUB, (a, b => c), b"\x87INT/SUB");

// Casting
instruction!(INT_TO_UINT, (a => b), b"\x89INT->UINT");
instruction!(UINT_TO_INT, (a => b), b"\x89UINT->INT");

// Comparison
instruction!(UINT_EQUALQ, (a, b => c), b"\x8BUINT/EQUAL?");
instruction!(UINT_GTQ, (a, b => c), b"\x88UINT/GT?");
instruction!(UINT_LTQ, (a, b => c), b"\x88UINT/LT?");
instruction!(INT_EQUALQ, (a, b => c), b"\x8AINT/EQUAL?");
instruction!(INT_GTQ, (a, b => c), b"\x87INT/GT?");
instruction!(INT_LTQ, (a, b => c), b"\x87INT/LT?");

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
    ($env: expr, $instruction: expr, $instruction_const: expr, $cmp: ident) => {{
        instruction_is!($env, $instruction, $instruction_const);
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
    ($env: expr, $instruction: expr, $instruction_const: expr, $cmp: ident) => {{
        instruction_is!($env, $instruction, $instruction_const);
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
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_instruction!(env, self.handle_uint_add(env, instruction, pid));
        try_instruction!(env, self.handle_uint_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int_add(env, instruction, pid));
        try_instruction!(env, self.handle_int_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int_to_uint(env, instruction, pid));
        try_instruction!(env, self.handle_uint_to_int(env, instruction, pid));
        try_instruction!(env, self.handle_uint_equalq(env, instruction, pid));
        try_instruction!(env, self.handle_uint_gtq(env, instruction, pid));
        try_instruction!(env, self.handle_uint_ltq(env, instruction, pid));
        try_instruction!(env, self.handle_int_equalq(env, instruction, pid));
        try_instruction!(env, self.handle_int_gtq(env, instruction, pid));
        try_instruction!(env, self.handle_int_ltq(env, instruction, pid));
        Err(Error::UnknownInstruction)
    }
}

impl<'a> Handler<'a> {

    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }


    #[inline]
    fn handle_uint_add(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        instruction_is!(env, instruction, UINT_ADD);
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

    fn handle_int_add(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        instruction_is!(env, instruction, INT_ADD);
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

    fn handle_int_sub(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        instruction_is!(env, instruction, INT_SUB);
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

    fn handle_int_to_uint(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId)
                          -> PassResult<'a> {
        instruction_is!(env, instruction, INT_TO_UINT);
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

    fn handle_uint_to_int(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId)
                          -> PassResult<'a> {
        instruction_is!(env, instruction, UINT_TO_INT);
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
    fn handle_uint_sub(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        instruction_is!(env, instruction, UINT_SUB);
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
    fn handle_uint_equalq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_EQUALQ, eq)
    }

    #[inline]
    fn handle_uint_gtq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_GTQ, gt)
    }

    #[inline]
    fn handle_uint_ltq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_LTQ, lt)
    }

    #[inline]
    fn handle_int_equalq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, instruction, INT_EQUALQ, eq)
    }

    #[inline]
    fn handle_int_gtq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, instruction, INT_GTQ, gt)
    }

    #[inline]
    fn handle_int_ltq(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        int_comparison!(env, instruction, INT_LTQ, lt)
    }

}