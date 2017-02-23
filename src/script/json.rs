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

word!(JSONQ, b"\x85JSON?");
word!(JSON_OBJECTQ, b"\x8CJSON/OBJECT?");
word!(JSON_STRINGQ, b"\x8CJSON/STRING?");
word!(JSON_NUMBERQ, b"\x8CJSON/NUMBER?");
word!(JSON_BOOLEANQ, b"\x8DJSON/BOOLEAN?");
word!(JSON_ARRAYQ, b"\x8BJSON/ARRAY?");
word!(JSON_NULLQ, b"\x8AJSON/NULL?");
word!(JSON_GET, b"\x88JSON/GET");
word!(JSON_SET, b"\x88JSON/SET");
word!(JSON_HASQ, b"\x89JSON/HAS?");
word!(JSON_STRING_TO, b"\x8dJSON/STRING->");
word!(JSON_TO_STRING, b"\x8dJSON/->STRING");

use super::{Env, EnvId, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE};
use serde_json as json;

use std::marker::PhantomData;

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

macro_rules! json_is_a {
    ($env: expr, $word: expr, $c: expr, { $t: ident }) => {{
        word_is!($env, $word, $c);
        let a = stack_pop!($env);

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::$t) => $env.push(STACK_TRUE),
            _ => $env.push(STACK_FALSE),
        }

        Ok(())
    }};
    ($env: expr, $word: expr, $c: expr, $t: ident) => {{
        word_is!($env, $word, $c);
        let a = stack_pop!($env);

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::$t(_)) => $env.push(STACK_TRUE),
            _ => $env.push(STACK_FALSE),
        }

        Ok(())
    }};
}


impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    #[inline]
    pub fn handle_jsonq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSONQ);
        let a = stack_pop!(env);

        match json::from_slice::<json::Value>(a) {
            Ok(_) => env.push(STACK_TRUE),
            Err(_) => env.push(STACK_FALSE),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_objectq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_OBJECTQ, Object)
    }

    #[inline]
    pub fn handle_json_stringq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_STRINGQ, String)
    }
    #[inline]
    pub fn handle_json_numberq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_NUMBERQ, Number)
    }

    #[inline]
    pub fn handle_json_booleanq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_BOOLEANQ, Bool)
    }

    #[inline]
    pub fn handle_json_arrayq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_ARRAYQ, Array)
    }

    #[inline]
    pub fn handle_json_nullq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        json_is_a!(env, word, JSON_NULLQ, { Null })
    }

    #[inline]
    pub fn handle_json_get(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSON_GET);

        let field = stack_pop!(env);
        let a = stack_pop!(env);

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field))
        };

        match json::from_slice::<json::Value>(a) {
            Ok(json::Value::Object(mut map)) => {
                match map.remove(&key) {
                    Some(val) => {
                        let s = val.to_string();
                        let val = alloc_and_write!(s.as_bytes(), env);
                        env.push(val);
                    }
                    None => {
                        return Err(error_invalid_value!(field))
                    }
                }
            }
            Ok(_) => return Err(error_invalid_value!(a)),
            Err(_) => return Err(error_invalid_value!(a)),
        }

        Ok(())
    }

    #[inline]
    pub fn handle_json_hasq(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSON_HASQ);

        let field = stack_pop!(env);
        let a = stack_pop!(env);

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field))
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
    pub fn handle_json_set(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSON_SET);

        let value = stack_pop!(env);
        let field = stack_pop!(env);
        let a = stack_pop!(env);

        let key = match String::from_utf8(Vec::from(field)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(field))
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
    pub fn handle_json_string_to(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSON_STRING_TO);

        let a = stack_pop!(env);

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
    pub fn handle_json_to_string(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, JSON_TO_STRING);

        let a = stack_pop!(env);

        let s = match String::from_utf8(Vec::from(a)) {
            Ok(k) => k,
            Err(_) => return Err(error_invalid_value!(a))
        };

        let str = json::Value::String(s).to_string();
        let val = alloc_and_write!(str.as_bytes(), env);
        env.push(val);

        Ok(())
    }

}
