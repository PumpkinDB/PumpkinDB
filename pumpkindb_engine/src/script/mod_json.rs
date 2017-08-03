// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # JSON
//!
//! This module handles everything JSON
//!

use core::convert::From;

instruction!(JSONQ, b"\x85JSON?");
instruction!(JSON_OBJECTQ, b"\x8CJSON/OBJECT?");
instruction!(JSON_STRINGQ, b"\x8CJSON/STRING?");
instruction!(JSON_NUMBERQ, b"\x8CJSON/NUMBER?");
instruction!(JSON_BOOLEANQ, b"\x8DJSON/BOOLEAN?");
instruction!(JSON_ARRAYQ, b"\x8BJSON/ARRAY?");
instruction!(JSON_NULLQ, b"\x8AJSON/NULL?");
instruction!(JSON_GET, b"\x88JSON/GET");
instruction!(JSON_SET, b"\x88JSON/SET");
instruction!(JSON_HASQ, b"\x89JSON/HAS?");
instruction!(JSON_STRING_TO, b"\x8dJSON/STRING->");
instruction!(JSON_TO_STRING, b"\x8dJSON/->STRING");

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE, TryInstruction};
use serde_json as json;

use std::marker::PhantomData;

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

macro_rules! json_is_a {
    ($env: expr, $instruction: expr, $c: expr, { $t: ident }) => {{
        return_unless_instructions_equal!($instruction, $c);
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::$t) => $env.push(STACK_TRUE),
            _ => $env.push(STACK_FALSE),
        }

        Ok(())
    }};
    ($env: expr, $instruction: expr, $c: expr, $t: ident) => {{
        return_unless_instructions_equal!($instruction, $c);
        let a = $env.pop().ok_or_else(|| error_empty_stack!())?;

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::$t(_)) => $env.push(STACK_TRUE),
            _ => $env.push(STACK_FALSE),
        }

        Ok(())
    }};
}

builtins!("mod_json.psc");

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_builtins(env, instruction, pid)
        .if_unhandled_try(|| self.handle_jsonq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_objectq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_stringq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_numberq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_booleanq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_arrayq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_nullq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_get(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_hasq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_set(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_string_to(env, instruction, pid))
        .if_unhandled_try(|| self.handle_json_to_string(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    handle_builtins!();

    #[inline]
    pub fn handle_jsonq(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSONQ);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        match json::from_slice::<json::Value>(a) {
            Ok(_) => env.push(STACK_TRUE),
            Err(_) => env.push(STACK_FALSE),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_objectq(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               _: EnvId)
                               -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_OBJECTQ, Object)
    }

    #[inline]
    pub fn handle_json_stringq(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               _: EnvId)
                               -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_STRINGQ, String)
    }
    #[inline]
    pub fn handle_json_numberq(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               _: EnvId)
                               -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_NUMBERQ, Number)
    }

    #[inline]
    pub fn handle_json_booleanq(&mut self,
                                env: &mut Env<'a>,
                                instruction: &'a [u8],
                                _: EnvId)
                                -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_BOOLEANQ, Bool)
    }

    #[inline]
    pub fn handle_json_arrayq(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              _: EnvId)
                              -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_ARRAYQ, Array)
    }

    #[inline]
    pub fn handle_json_nullq(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             _: EnvId)
                             -> PassResult<'a> {
        json_is_a!(env, instruction, JSON_NULLQ, {
            Null
        })
    }

    #[inline]
    pub fn handle_json_get(&mut self,
                           env: &mut Env<'a>,
                           instruction: &'a [u8],
                           _: EnvId)
                           -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSON_GET);

        let field = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field)),
        };

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::Object(mut map)) => {
                match map.remove(&key) {
                    Some(val) => {
                        let s = val.to_string();
                        let val = alloc_and_write!(s.as_bytes(), env);
                        env.push(val);
                    }
                    None => return Err(error_invalid_value!(field)),
                }
            }
            Ok(_) => return Err(error_invalid_value!(a)),
            Err(_) => return Err(error_invalid_value!(a)),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_hasq(&mut self,
                            env: &mut Env<'a>,
                            instruction: &'a [u8],
                            _: EnvId)
                            -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSON_HASQ);

        let field = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field)),
        };

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::Object(map)) => {
                if map.contains_key(&key) {
                    env.push(STACK_TRUE);
                } else {
                    env.push(STACK_FALSE);
                }
            }
            Ok(_) => return Err(error_invalid_value!(a)),
            Err(_) => return Err(error_invalid_value!(a)),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_set(&mut self,
                           env: &mut Env<'a>,
                           instruction: &'a [u8],
                           _: EnvId)
                           -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSON_SET);

        let value = env.pop().ok_or_else(|| error_empty_stack!())?;
        let field = env.pop().ok_or_else(|| error_empty_stack!())?;
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field)),
        };

        let value = match json::from_slice::<json::Value>(value) {
            Ok(v) => v,
            Err(_) => return Err(error_invalid_value!(value)),
        };

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::Object(mut map)) => {
                map.insert(key, value);
                let s = json::Value::Object(map).to_string();
                let val = alloc_and_write!(s.as_bytes(), env);
                env.push(val);
            }
            Ok(_) => return Err(error_invalid_value!(a)),
            Err(_) => return Err(error_invalid_value!(a)),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_string_to(&mut self,
                                 env: &mut Env<'a>,
                                 instruction: &'a [u8],
                                 _: EnvId)
                                 -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSON_STRING_TO);

        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::String(val)) => {
                let val = alloc_and_write!(val.as_bytes(), env);
                env.push(val);
            }
            Ok(_) => return Err(error_invalid_value!(a)),
            Err(_) => return Err(error_invalid_value!(a)),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_to_string(&mut self,
                                 env: &mut Env<'a>,
                                 instruction: &'a [u8],
                                 _: EnvId)
                                 -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, JSON_TO_STRING);

        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        let s = match String::from_utf8(Vec::from(a)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(a)),
        };

        let str = json::Value::String(s).to_string();
        let val = alloc_and_write!(str.as_bytes(), env);
        env.push(val);

        Ok(())
    }
}
