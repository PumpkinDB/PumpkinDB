// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

instruction!(UUID_V4, b"\x87UUID/V4");
instruction!(UUID_V5, b"\x87UUID/V5");
instruction!(UUID_TO_STRING, b"\x8dUUID/->STRING");
instruction!(UUID_STRING_TO, b"\x8dUUID/STRING->");

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, TryInstruction};

use core::str::FromStr;
use uuid::Uuid;
use std::marker::PhantomData;
use std::str;

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_uuid_v4(env, instruction, pid)
        .if_unhandled_try(|| self.handle_uuid_v5(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uuid_to_string(env, instruction, pid))
        .if_unhandled_try(|| self.handle_uuid_string_to(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    pub fn handle_uuid_v4(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UUID_V4);
        let uuid = Uuid::new_v4();
        let mut slice = alloc_slice!(16, env);
        slice.copy_from_slice(uuid.as_bytes());
        env.push(slice);
        Ok(())
    }

    #[inline]
    pub fn handle_uuid_v5(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UUID_V5);
        let name_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
        if let Ok(name) = str::from_utf8(name_bytes) {
            let ns_uuid_bytes = env.pop().ok_or_else(|| error_empty_stack!())?;
            if let Ok(ns_uuid) = Uuid::from_bytes(ns_uuid_bytes) {
                let uuid = Uuid::new_v5(&ns_uuid, name);
                let mut slice = alloc_slice!(16, env);
                slice.copy_from_slice(uuid.as_bytes());
                env.push(slice);
                Ok(())
            } else {
                Err(error_invalid_value!(ns_uuid_bytes))
            }
        } else {
            Err(error_invalid_value!(name_bytes))
        }
    }


    #[inline]
    pub fn handle_uuid_to_string(&mut self,
                                 env: &mut Env<'a>,
                                 instruction: &'a [u8],
                                 _: EnvId)
                                 -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UUID_TO_STRING);

        let top = env.pop().ok_or_else(|| error_empty_stack!())?;

        if let Ok(uuid) = Uuid::from_bytes(top) {
            let str = uuid.hyphenated().to_string();
            let val = alloc_and_write!(str.as_bytes(), env);
            env.push(val);

            Ok(())
        } else {
            Err(error_invalid_value!(top))
        }
    }

    #[inline]
    pub fn handle_uuid_string_to(&mut self,
                                 env: &mut Env<'a>,
                                 instruction: &'a [u8],
                                 _: EnvId)
                                 -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, UUID_STRING_TO);

        let top = env.pop().ok_or_else(|| error_empty_stack!())?;

        if let Ok(uuid_str) = str::from_utf8(top) {
            if let Ok(uuid) = Uuid::from_str(uuid_str) {
                let bytes = alloc_and_write!(uuid.as_bytes(), env);
                env.push(bytes);

                Ok(())
            } else {
                Err(error_invalid_value!(top))
            }
        } else {
            Err(error_invalid_value!(top))
        }
    }
}
