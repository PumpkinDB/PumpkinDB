// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE};

use ::pumpkinscript::{Packable, Unpackable};

use std::marker::PhantomData;

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use num_bigint::{BigUint, BigInt, Sign};
use num_traits::Signed;
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


pub fn bytes_to_bigint(bytes: &[u8]) -> Option<BigInt> {
    match bytes[0] {
            0x00 => Some(Sign::Minus),
            0x01 => Some(Sign::Plus),
            _ => None
        }
        .and_then(|sign| Some(
                {
                    let mut bytes: Vec<u8> = Vec::from(&bytes[1..]);
                    if sign == Sign::Plus { 
                        BigInt::from_bytes_be(sign, bytes.as_slice())
                    } else {
                        for i in 0..bytes.len() {
                            //flip.push(!byte);
                            bytes[i] = !bytes[i];
                        }
                        let mut nextbit = true;
                        for i in (0..bytes.len()).rev() {
                            bytes[i] = match bytes[i].checked_add(1) {
                                Some(v) => {
                                    nextbit = false;
                                    v
                                },
                                None => 0,
                            };
                            if !nextbit {
                                break;
                            }
                        }
                        BigInt::from_bytes_be(sign, &bytes)
                    }
                }
                ))
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
        instruction_is!($instruction, $instruction_const);
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
        instruction_is!($instruction, $instruction_const);
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


macro_rules! no_endianness_sized_uint_op {
    ($env: expr, $read_op: ident, $op: ident, $write_op: ident) => {{
        let mut b = stack_pop!($env);
        let mut a = stack_pop!($env);

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
        let b = stack_pop!($env);
        let a = stack_pop!($env);

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
        let mut b = stack_pop!($env);
        let mut a = stack_pop!($env);

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
        let b = stack_pop!($env);
        let a = stack_pop!($env);

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

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
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
        try_instruction!(env, self.handle_uint8_add(env, instruction, pid));
        try_instruction!(env, self.handle_uint8_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int8_add(env, instruction, pid));
        try_instruction!(env, self.handle_int8_sub(env, instruction, pid));
        try_instruction!(env, self.handle_uint16_add(env, instruction, pid));
        try_instruction!(env, self.handle_uint16_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int16_add(env, instruction, pid));
        try_instruction!(env, self.handle_int16_sub(env, instruction, pid));
        try_instruction!(env, self.handle_uint32_add(env, instruction, pid));
        try_instruction!(env, self.handle_uint32_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int32_add(env, instruction, pid));
        try_instruction!(env, self.handle_int32_sub(env, instruction, pid));
        try_instruction!(env, self.handle_uint64_add(env, instruction, pid));
        try_instruction!(env, self.handle_uint64_sub(env, instruction, pid));
        try_instruction!(env, self.handle_int64_add(env, instruction, pid));
        try_instruction!(env, self.handle_int64_sub(env, instruction, pid));
        try_instruction!(env, self.handle_f32_add(env, instruction, pid));
        try_instruction!(env, self.handle_f32_sub(env, instruction, pid));
        try_instruction!(env, self.handle_f64_add(env, instruction, pid));
        try_instruction!(env, self.handle_f64_sub(env, instruction, pid));
        Err(Error::UnknownInstruction)
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
        instruction_is!(instruction, UINT_ADD);
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

    fn handle_int_add(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        instruction_is!(instruction, INT_ADD);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_int = bytes_to_bigint(a);
        let b_int = bytes_to_bigint(b);

        if a_int == None {
            return Err(error_invalid_value!(a));
        }
        if b_int == None {
            return Err(error_invalid_value!(b));
        }

        let c_int = a_int.unwrap().add(b_int.unwrap());

        let mut bytes = if c_int.is_negative() {
            vec![0x00]
        } else {
            vec![0x01]
        };
        let (_, mut c_bytes) = c_int.to_bytes_be();
        if c_int.is_negative() {
            for i in 0..c_bytes.len() {
                    c_bytes[i] = !c_bytes[i];
                }
                let mut nextbit = true;
                for i in (0..c_bytes.len()).rev() {
                    c_bytes[i] =  match c_bytes[i].checked_add(1) {
                        Some(v) => {
                            nextbit = false;
                            v
                        },
                        None => 0,
                    };
                    if !nextbit {
                        break;
                    }
                }
        }

        bytes.extend_from_slice(&c_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_sub(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        instruction_is!(instruction, INT_SUB);
        let a = stack_pop!(env);
        let b = stack_pop!(env);

        let a_int = bytes_to_bigint(a);
        let b_int = bytes_to_bigint(b);

        if a_int == None {
            return Err(error_invalid_value!(a));
        }
        if b_int == None {
            return Err(error_invalid_value!(b));
        }

        let c_int = b_int.unwrap().sub(a_int.unwrap());

        let mut bytes = if c_int.is_negative() {
            vec![0x00]
        } else {
            vec![0x01]
        };
        let (_, mut c_bytes) = c_int.to_bytes_be();
        if c_int.is_negative() {
            for i in 0..c_bytes.len() {
                    c_bytes[i] = !c_bytes[i];
                }
                let mut nextbit = true;
                for i in (0..c_bytes.len()).rev() {
                    c_bytes[i] =  match c_bytes[i].checked_add(1) {
                        Some(v) => {
                            nextbit = false;
                            v
                        },
                        None => 0,
                    };
                    if !nextbit {
                        break;
                    }
                }
        }

        bytes.extend_from_slice(&c_bytes);
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);
        Ok(())
    }

    fn handle_int_to_uint(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        instruction_is!(instruction, INT_TO_UINT);
        let a = stack_pop!(env);
        let a_int = bytes_to_bigint(a);

        if a_int == None {
            return Err(error_invalid_value!(a));
        }

        match a_int.unwrap().to_biguint() {
            Some(a_uint) => {
                let a_bytes = a_uint.to_bytes_be();
                let slice = alloc_and_write!(a_bytes.as_slice(), env);
                env.push(slice);
                Ok(())
            }
            None => Err(error_invalid_value!(a)),
        }
    }

    fn handle_uint_to_int(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        instruction_is!(instruction, UINT_TO_INT);
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
    fn handle_uint_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT_SUB);
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
        instruction_is!(instruction, UINT8_ADD);
        no_endianness_sized_uint_op!(env, read_u8, checked_add, write_u8)
    }

    #[inline]
    fn handle_uint8_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT8_SUB);
        no_endianness_sized_uint_op!(env, read_u8, checked_sub, write_u8)
    }

    #[inline]
    fn handle_int8_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT8_ADD);
        no_endianness_sized_int_op!(env, read_i8, checked_add, write_i8)
    }

    #[inline]
    fn handle_int8_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT8_SUB);
        no_endianness_sized_int_op!(env, read_i8, checked_sub, write_i8)
    }

    #[inline]
    fn handle_uint16_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT16_ADD);
        sized_uint_op!(env, read_u16, checked_add, write_u16)
    }

    #[inline]
    fn handle_uint16_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT16_SUB);
        sized_uint_op!(env, read_u16, checked_sub, write_u16)
    }

    #[inline]
    fn handle_int16_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT16_ADD);
        sized_int_op!(env, read_i16, checked_add, write_i16)
    }

    #[inline]
    fn handle_int16_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT16_SUB);
        sized_int_op!(env, read_i16, checked_sub, write_i16)
    }

    #[inline]
    fn handle_uint32_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT32_ADD);
        sized_uint_op!(env, read_u32, checked_add, write_u32)
    }

    #[inline]
    fn handle_uint32_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT32_SUB);
        sized_uint_op!(env, read_u32, checked_sub, write_u32)
    }

    #[inline]
    fn handle_int32_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT32_ADD);
        sized_int_op!(env, read_i32, checked_add, write_i32)
    }

    #[inline]
    fn handle_int32_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT32_SUB);
        sized_int_op!(env, read_i32, checked_sub, write_i32)
    }

    #[inline]
    fn handle_uint64_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT64_ADD);
        sized_uint_op!(env, read_u64, checked_add, write_u64)
    }

    #[inline]
    fn handle_uint64_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, UINT64_SUB);
        sized_uint_op!(env, read_u64, checked_sub, write_u64)
    }

    #[inline]
    fn handle_int64_add(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT64_ADD);
        sized_int_op!(env, read_i64, checked_add, write_i64)
    }

    #[inline]
    fn handle_int64_sub(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        instruction_is!(instruction, INT64_SUB);
        sized_int_op!(env, read_i64, checked_sub, write_i64)
    }
    
    #[inline]
    fn handle_f32_add(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        instruction_is!(instruction, F32_ADD);
        let a_bytes = stack_pop!(env);
        let a: f32 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
            
        let b_bytes = stack_pop!(env);
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
        instruction_is!(instruction, F32_SUB);
        let a_bytes = stack_pop!(env);
        let a: f32 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = stack_pop!(env);
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
        instruction_is!(instruction, F64_ADD);
        let a_bytes = stack_pop!(env);
        let a: f64 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = stack_pop!(env);
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
        instruction_is!(instruction, F64_SUB);
        let a_bytes = stack_pop!(env);
        let a: f64 = a_bytes.unpack().ok_or(error_invalid_value!(a_bytes))?;
        
        let b_bytes = stack_pop!(env);
        let b: f64 = b_bytes.unpack().ok_or(error_invalid_value!(b_bytes))?;
        
        let bytes = (b - a).pack();
        let slice = alloc_and_write!(bytes.as_slice(), env);
        env.push(slice);

        Ok(())
    }

}
