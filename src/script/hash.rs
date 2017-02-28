// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Hashing primitives
//!
//! This module handles hashing data
//!
word!(HASH_SHA1, b"\x89HASH/SHA1");
word!(HASH_SHA224, b"\x8BHASH/SHA224");
word!(HASH_SHA256, b"\x8BHASH/SHA256");
word!(HASH_SHA384, b"\x8BHASH/SHA384");
word!(HASH_SHA512, b"\x8BHASH/SHA512");
word!(HASH_SHA512_224, b"\x8FHASH/SHA512-224");
word!(HASH_SHA512_256, b"\x8FHASH/SHA512-256");

/*
 * `Sha224`, which is the 32-bit `Sha256` algorithm with the result truncated to 224 bits.
 * `Sha256`, which is the 32-bit `Sha256` algorithm.
 * `Sha384`, which is the 64-bit `Sha512` algorithm with the result truncated to 384 bits.
 * `Sha512`, which is the 64-bit `Sha512` algorithm.
 * `Sha512Trunc224`, which is the 64-bit `Sha512` algorithm with the result truncated to 224 bits.
 * `Sha512Trunc256`, which is the 64-bit `Sha512` algorithm with the result truncated to 256 bits.
*/

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK, offset_by_size};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use crypto::sha2::*;

use std::marker::PhantomData;

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>
}

macro_rules! hash_word {
    ($name : ident, $constant: ident, $i: ident, $size: expr) => {
    #[inline]
    pub fn $name(&mut self, env: &mut Env<'a>, word: &'a [u8], _: EnvId) -> PassResult<'a> {
        word_is!(env, word, $constant);
        let a = stack_pop!(env);
        let mut hasher = $i::new();
        hasher.input(a);
        let mut slice = alloc_slice!($size, env);
        hasher.result(&mut slice);
        env.push(slice);
        Ok(())
    }
    };
}

impl<'a> Module<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_hash_sha1(env, word, pid));
        try_word!(env, self.handle_hash_sha224(env, word, pid));
        try_word!(env, self.handle_hash_sha256(env, word, pid));
        try_word!(env, self.handle_hash_sha384(env, word, pid));
        try_word!(env, self.handle_hash_sha512(env, word, pid));
        try_word!(env, self.handle_hash_sha512_224(env, word, pid));
        try_word!(env, self.handle_hash_sha512_256(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler { phantom: PhantomData }
    }

    hash_word!(handle_hash_sha1, HASH_SHA1, Sha1, 20);
    hash_word!(handle_hash_sha224, HASH_SHA224, Sha224, 28);
    hash_word!(handle_hash_sha256, HASH_SHA256, Sha256, 32);
    hash_word!(handle_hash_sha384, HASH_SHA384, Sha384, 48);
    hash_word!(handle_hash_sha512, HASH_SHA512, Sha512, 64);
    hash_word!(handle_hash_sha512_224, HASH_SHA512_224, Sha512Trunc224, 28);
    hash_word!(handle_hash_sha512_256, HASH_SHA512_256, Sha512Trunc256, 32);

}
