// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, TryInstruction};

use ::pumpkinscript::Packable;
use core::str::FromStr;
use std::marker::PhantomData;
use num_bigint::{BigUint, BigInt};

instruction!(STRING_TO_UINT, b"\x8dSTRING/->UINT");
instruction!(STRING_TO_INT, b"\x8cSTRING/->INT");
instruction!(STRING_TO_UINT8, b"\x8eSTRING/->UINT8");
instruction!(STRING_TO_INT8 ,b"\x8dSTRING/->INT8");
instruction!(STRING_TO_UINT16, b"\x8fSTRING/->UINT16");
instruction!(STRING_TO_INT16, b"\x8eSTRING/->INT16");
instruction!(STRING_TO_UINT32, b"\x8fSTRING/->UINT32");
instruction!(STRING_TO_INT32, b"\x8eSTRING/->INT32");
instruction!(STRING_TO_UINT64, b"\x8fSTRING/->UINT64");
instruction!(STRING_TO_INT64, b"\x8eSTRING/->INT64");
instruction!(STRING_TO_F32, b"\x8cSTRING/->F32");
instruction!(STRING_TO_F64, b"\x8cSTRING/->F64");

macro_rules! to_sized {
    ($env: expr, $type: ident) => {{
        let a_bytes = $env.pop().ok_or_else(|| error_empty_stack!())?;
        let s = String::from_utf8(Vec::from(a_bytes)).or(Err(error_invalid_value!(a_bytes)))?;
        let a = $type::from_str(&s).or(Err(error_invalid_value!(a_bytes)))?;
        a.pack()
    }}
}

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_to_uint(env, instruction, pid)
        .if_unhandled_try(|| self.handle_to_int(env, instruction, pid))
        .if_unhandled_try(|| self.handle_to_sized_num(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }
    
    #[inline]
    pub fn handle_to_uint(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               _: EnvId)
                               -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, STRING_TO_UINT);

        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let s = String::from_utf8(Vec::from(a_bytes)).or(Err(error_invalid_value!(a_bytes)))?;
        let a: BigUint = BigUint::from_str(&s).or(Err(error_invalid_value!(a_bytes)))?;

        let slice = alloc_and_write!(a.pack().as_slice(), env);
        env.push(slice);

        Ok(())
    }

    pub fn handle_to_int(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, STRING_TO_INT);

        let a_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        let s = String::from_utf8(Vec::from(a_bytes)).or(Err(error_invalid_value!(a_bytes)))?;
        let a: BigInt = BigInt::from_str(&s).or(Err(error_invalid_value!(a_bytes)))?;

        let slice = alloc_and_write!(a.pack().as_slice(), env);
        env.push(slice);

        Ok(())
    }
    
    #[inline]
    pub fn handle_to_sized_num(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               _: EnvId)
                                 -> PassResult<'a> {
        
        let a = match instruction {
            STRING_TO_UINT8   => to_sized!(env, u8),
            STRING_TO_INT8    => to_sized!(env, i8),
            STRING_TO_UINT16  => to_sized!(env, u16),
            STRING_TO_INT16   => to_sized!(env, i16),
            STRING_TO_UINT32  => to_sized!(env, u32),
            STRING_TO_INT32   => to_sized!(env, i32),
            STRING_TO_UINT64  => to_sized!(env, u64),
            STRING_TO_INT64   => to_sized!(env, i64),
            STRING_TO_F32     => to_sized!(env, f32),
            STRING_TO_F64     => to_sized!(env, f64),
            
            _ => return Err(Error::UnknownInstruction),
        };        

        let slice = alloc_and_write!(a.as_slice(), env);
        env.push(slice);

        Ok(())
    }

}
