// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE, TryInstruction};

use ::pumpkinscript::{Packable, Unpackable};

use std::marker::PhantomData;

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use num_bigint::{BigUint, BigInt};
use core::ops::{Add, Sub};

// Category: arithmetics
instruction!(UINT_ADD, (a, b => c), b"\x88UINT/ADD");
instruction!(UINT_SUB, (a, b => c), b"\x88UINT/SUB");
instruction!(INT_ADD, (a, b => c), b"\x87INT/ADD");
instruction!(INT_SUB, (a, b => c), b"\x87INT/SUB");

instruction!(UINT8_ADD, (a, b => c), b"\x89UINT8/ADD");
instruction!(UINT8_SUB, (a, b => c), b"\x89UINT8/SUB");
instruction!(INT8_ADD, (a, b => c), b"\x88INT8/ADD");
instruction!(INT8_SUB, (a, b => c), b"\x88INT8/SUB");

instruction!(UINT16_ADD, (a, b => c), b"\x8aUINT16/ADD");
instruction!(UINT16_SUB, (a, b => c), b"\x8aUINT16/SUB");
instruction!(INT16_ADD, (a, b => c), b"\x89INT16/ADD");
instruction!(INT16_SUB, (a, b => c), b"\x89INT16/SUB");

instruction!(UINT32_ADD, (a, b => c), b"\x8aUINT32/ADD");
instruction!(UINT32_SUB, (a, b => c), b"\x8aUINT32/SUB");
instruction!(INT32_ADD, (a, b => c), b"\x89INT32/ADD");
instruction!(INT32_SUB, (a, b => c), b"\x89INT32/SUB");

instruction!(UINT64_ADD, (a, b => c), b"\x8aUINT64/ADD");
instruction!(UINT64_SUB, (a, b => c), b"\x8aUINT64/SUB");
instruction!(INT64_ADD, (a, b => c), b"\x89INT64/ADD");
instruction!(INT64_SUB, (a, b => c), b"\x89INT64/SUB");

instruction!(F32_ADD, b"\x87F32/ADD");
instruction!(F32_SUB, b"\x87F32/SUB");
instruction!(F64_ADD, b"\x87F64/ADD");
instruction!(F64_SUB, b"\x87F64/SUB");

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

// Stringify
instruction!(UINT_TO_STRING, b"\x8dUINT/->STRING");
instruction!(INT_TO_STRING, b"\x8cINT/->STRING");

instruction!(UINT8_TO_STRING, b"\x8eUINT8/->STRING");
instruction!(UINT16_TO_STRING, b"\x8fUINT16/->STRING");
instruction!(UINT32_TO_STRING, b"\x8fUINT32/->STRING");
instruction!(UINT64_TO_STRING, b"\x8fUINT64/->STRING");

instruction!(INT8_TO_STRING, b"\x8dINT8/->STRING");
instruction!(INT16_TO_STRING, b"\x8eINT16/->STRING");
instruction!(INT32_TO_STRING, b"\x8eINT32/->STRING");
instruction!(INT64_TO_STRING, b"\x8eINT64/->STRING");

instruction!(F32_TO_STRING, b"\x8cF32/->STRING");
instruction!(F64_TO_STRING, b"\x8cF64/->STRING");

macro_rules! uint_comparison {
    ($env: expr, $instruction: expr, $instruction_const: expr, $cmp: ident) => {{
        return_unless_instructions_equal!($instruction, $instruction_const);
        let b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_: BigUint = a.unpack().ok_or(error_invalid_value!(a))?;
        let b_: BigUint = b.unpack().ok_or(error_invalid_value!(b))?;

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
        return_unless_instructions_equal!($instruction, $instruction_const);
        let b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_: BigInt = a.unpack().ok_or(error_invalid_value!(a))?;
        let b_: BigInt = b.unpack().ok_or(error_invalid_value!(b))?;

        if a_.$cmp(&b_) {
            $env.push(STACK_TRUE);
        } else {
            $env.push(STACK_FALSE);
        }
        Ok(())
    }};
}


macro_rules! no_endianness_sized_uint_op {
    ($env: expr, $read_op: ident, $op: ident, $write_op: ident) => {{
        let mut b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let mut a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_int = match a.$read_op() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(a)),
        };

        let b_int = match b.$read_op() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(b)),
        };

        let c_int = match a_int.$op(b_int) {
            Some(v) => v,
            None => return Err(error_invalid_value!(a)),
        };

        let mut c_bytes = vec![];
        match c_bytes.$write_op(c_int) {
            Ok(_) => {},
            Err(_) => return Err(error_invalid_value!(a)),
        }

        let slice = alloc_and_write!(c_bytes.as_slice(), $env);
        $env.push(slice);
        Ok(())
    }};
}

macro_rules! no_endianness_sized_int_op {
    ($env: expr, $read_op: ident, $op: ident, $write_op: ident) => {{
        let b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let mut a = Vec::from(a);
        a[0] ^= 1u8 << 7;

        let a_int = match a.as_slice().$read_op() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(a)),
        };

        let mut b = Vec::from(b);
        b[0] ^= 1u8 << 7;

        let b_int = match b.as_slice().$read_op() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(b)),
        };

        let c_int = match a_int.$op(b_int) {
            Some(v) => v,
            None => return Err(error_invalid_value!(a)),
        };

        let mut c_bytes = vec![];
        match c_bytes.$write_op(c_int) {
            Ok(_) => {},
            Err(_) => return Err(error_invalid_value!(a)),
        }
        
        c_bytes[0] ^= 1u8 << 7;

        let slice = alloc_and_write!(c_bytes.as_slice(), $env);
        $env.push(slice);
        Ok(())
    }};
}

macro_rules! sized_uint_op {
    ($env: expr, $read_op: ident, $op: ident, $write_op: ident) => {{
        let mut b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let mut a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_int = match a.$read_op::<BigEndian>() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(a)),
        };

        let b_int = match b.$read_op::<BigEndian>() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(b)),
        };

        let c_int = match a_int.$op(b_int) {
            Some(v) => v,
            None => return Err(error_invalid_value!(a)),
        };

        let mut c_bytes = vec![];
        match c_bytes.$write_op::<BigEndian>(c_int) {
            Ok(_) => {},
            Err(_) => return Err(error_invalid_value!(a)),
        }

        let slice = alloc_and_write!(c_bytes.as_slice(), $env);
        $env.push(slice);
        Ok(())
    }};
}

macro_rules! sized_int_op {
    ($env: expr, $read_op: ident, $op: ident, $write_op: ident) => {{
        let b = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let mut a = Vec::from(a);
        a[0] ^= 1u8 << 7;

        let a_int = match a.as_slice().$read_op::<BigEndian>() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(a)),
        };

        let mut b = Vec::from(b);
        b[0] ^= 1u8 << 7;

        let b_int = match b.as_slice().$read_op::<BigEndian>() {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(b)),
        };

        let c_int = match a_int.$op(b_int) {
            Some(v) => v,
            None => return Err(error_invalid_value!(a)),
        };

        let mut c_bytes = vec![];
        match c_bytes.$write_op::<BigEndian>(c_int) {
            Ok(_) => {},
            Err(_) => return Err(error_invalid_value!(a)),
        }

        c_bytes[0] ^= 1u8 << 7;

        let slice = alloc_and_write!(c_bytes.as_slice(), $env);
        $env.push(slice);
        Ok(())
    }};
}

macro_rules! to_string {
    ($env: expr, $type: ident) => {{
        let a_bytes = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: $type = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        format!("{}", a)
    }}
}


pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_uint_add(env, instruction, pid)
        .if_unhandled_try(|| self.handle_uint_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_to_uint(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint_to_int(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint_equalq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint_gtq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint_ltq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_equalq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_gtq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_ltq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint8_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint8_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int8_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int8_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint16_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint16_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int16_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int16_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint32_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint32_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int32_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int32_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint64_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uint64_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int64_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int64_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_f32_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_f32_sub(env, instruction, pid))
        .if_unhandled_try(|| self.handle_f64_add(env, instruction, pid))
        .if_unhandled_try(|| self.handle_f64_sub(env, instruction, pid))

        .if_unhandled_try(|| self.handle_uint_to_string(env, instruction, pid))
        .if_unhandled_try(|| self.handle_int_to_string(env, instruction, pid))
        .if_unhandled_try(|| self.handle_to_string(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }


    #[inline]
    fn handle_uint_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT_ADD);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_int: BigUint = a.unpack().ok_or(error_invalid_value!(a))?;
        let b_int: BigUint = b.unpack().ok_or(error_invalid_value!(b))?;

        let c_int = a_int.add(b_int);

        let slice = alloc_and_write!(c_int.pack().as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_add(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT_ADD);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_int: BigInt = a.unpack().ok_or(error_invalid_value!(a))?;
        let b_int: BigInt = b.unpack().ok_or(error_invalid_value!(b))?;

        let c_int = a_int.add(b_int);

        let slice = alloc_and_write!(c_int.pack().as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_sub(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT_SUB);
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let a_int: BigInt = a.unpack().ok_or(error_invalid_value!(a))?;
        let b_int: BigInt = b.unpack().ok_or(error_invalid_value!(b))?;

        let c_int = a_int.sub(b_int);

        let slice = alloc_and_write!(c_int.pack().as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_to_uint(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT_TO_UINT);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a_int : BigInt = a.unpack().ok_or(error_invalid_value!(a))?;

        let a_uint = a_int.to_biguint().ok_or(error_invalid_value!(a))?;
        let slice = alloc_and_write!(a_uint.pack().as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_uint_to_int(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT_TO_INT);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a_uint = BigUint::from_bytes_be(a);

        let mut bytes = vec![0x01];
        let a_bytes = a_uint.to_bytes_be();
        bytes.extend_from_slice(&a_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);

        env.push(slice);
        Ok(())
    }

    #[inline]
    fn handle_uint_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT_SUB);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

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
    fn handle_uint_equalq(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_EQUALQ, eq)
    }

    #[inline]
    fn handle_uint_gtq(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_GTQ, gt)
    }

    #[inline]
    fn handle_uint_ltq(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        uint_comparison!(env, instruction, UINT_LTQ, lt)
    }

    #[inline]
    fn handle_int_equalq(&mut self,
                         env: &mut Env<'a>,
                         instruction: &'a [u8],
                         _: EnvId)
                         -> PassResult<'a> {
        int_comparison!(env, instruction, INT_EQUALQ, eq)
    }

    #[inline]
    fn handle_int_gtq(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        int_comparison!(env, instruction, INT_GTQ, gt)
    }

    #[inline]
    fn handle_int_ltq(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        int_comparison!(env, instruction, INT_LTQ, lt)
    }

    #[inline]
    fn handle_uint8_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT8_ADD);
        no_endianness_sized_uint_op!(env, read_u8, checked_add, write_u8)
    }

    #[inline]
    fn handle_uint8_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT8_SUB);
        no_endianness_sized_uint_op!(env, read_u8, checked_sub, write_u8)
    }

    #[inline]
    fn handle_int8_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT8_ADD);
        no_endianness_sized_int_op!(env, read_i8, checked_add, write_i8)
    }

    #[inline]
    fn handle_int8_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT8_SUB);
        no_endianness_sized_int_op!(env, read_i8, checked_sub, write_i8)
    }

    #[inline]
    fn handle_uint16_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT16_ADD);
        sized_uint_op!(env, read_u16, checked_add, write_u16)
    }

    #[inline]
    fn handle_uint16_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT16_SUB);
        sized_uint_op!(env, read_u16, checked_sub, write_u16)
    }

    #[inline]
    fn handle_int16_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT16_ADD);
        sized_int_op!(env, read_i16, checked_add, write_i16)
    }

    #[inline]
    fn handle_int16_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT16_SUB);
        sized_int_op!(env, read_i16, checked_sub, write_i16)
    }

    #[inline]
    fn handle_uint32_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT32_ADD);
        sized_uint_op!(env, read_u32, checked_add, write_u32)
    }

    #[inline]
    fn handle_uint32_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT32_SUB);
        sized_uint_op!(env, read_u32, checked_sub, write_u32)
    }

    #[inline]
    fn handle_int32_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT32_ADD);
        sized_int_op!(env, read_i32, checked_add, write_i32)
    }

    #[inline]
    fn handle_int32_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT32_SUB);
        sized_int_op!(env, read_i32, checked_sub, write_i32)
    }

    #[inline]
    fn handle_uint64_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT64_ADD);
        sized_uint_op!(env, read_u64, checked_add, write_u64)
    }

    #[inline]
    fn handle_uint64_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT64_SUB);
        sized_uint_op!(env, read_u64, checked_sub, write_u64)
    }

    #[inline]
    fn handle_int64_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT64_ADD);
        sized_int_op!(env, read_i64, checked_add, write_i64)
    }

    #[inline]
    fn handle_int64_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT64_SUB);
        sized_int_op!(env, read_i64, checked_sub, write_i64)
    }
    
    #[inline]
    fn handle_f32_add(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, F32_ADD);
        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: f32 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
            
        let b_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b: f32 = b_bytes.unpack().ok_or(error_invalid_value!(b_bytes))?;

        let bytes = (a + b).pack();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        
        Ok(())                  
    }

    #[inline]
    fn handle_f32_sub(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, F32_SUB);
        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: f32 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b: f32 = b_bytes.unpack().ok_or(error_invalid_value!(b_bytes))?;
        
        let bytes = (b - a).pack();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_f64_add(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, F64_ADD);
        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: f64 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b: f64 = b_bytes.unpack().ok_or(error_invalid_value!(b_bytes))?;
        
        let bytes = (a + b).pack();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        
        Ok(())                  
    }

    #[inline]
    fn handle_f64_sub(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, F64_SUB);
        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: f64 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b: f64 = b_bytes.unpack().ok_or(error_invalid_value!(b_bytes))?;
        
        let bytes = (b - a).pack();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);

        Ok(())
    }

    #[inline]
    fn handle_uint_to_string(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UINT_TO_STRING);

        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: BigUint = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;

        let s = format!("{}", a);
        let val = alloc_and_write!(s.as_bytes(), env);
        env.push(val);

        Ok(())
    }

    #[inline]
    fn handle_int_to_string(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             _: EnvId)
                             -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, INT_TO_STRING);
        
        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a: BigInt = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;

        let s = format!("{}", a);
        let val = alloc_and_write!(s.as_bytes(), env);
        env.push(val);

        Ok(())
    }

    #[inline]
    fn handle_to_string(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {

        let s = match instruction {
            UINT8_TO_STRING        => to_string!(env, u8),
            INT8_TO_STRING         => to_string!(env, i8),
            UINT16_TO_STRING       => to_string!(env, u16),
            INT16_TO_STRING        => to_string!(env, i16),
            UINT32_TO_STRING       => to_string!(env, u32),
            INT32_TO_STRING        => to_string!(env, i32),
            UINT64_TO_STRING       => to_string!(env, u64),
            INT64_TO_STRING        => to_string!(env, i64),
            F32_TO_STRING          => to_string!(env, f32),
            F64_TO_STRING          => to_string!(env, f64),
            
            _ => return Err(Error::UnknownInstruction),
        };
        
        let val = alloc_and_write!(s.as_bytes(), env);
        env.push(val);


        Ok(())
    }
}
