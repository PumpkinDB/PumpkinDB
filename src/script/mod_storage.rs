// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Storage
//!
//! This module handles all words and state related to handling storage
//! capabilities
//!
use lmdb;
use lmdb::traits::LmdbResultExt;
use std::mem;
use std::error::Error as StdError;
use super::{Env, EnvId, Module, PassResult, Error, STACK_TRUE, STACK_FALSE, offset_by_size,
            ERROR_EMPTY_STACK, ERROR_INVALID_VALUE, ERROR_DUPLICATE_KEY, ERROR_NO_TX,
            ERROR_UNKNOWN_KEY, ERROR_DATABASE};
use core::ops::Deref;
use byteorder::{BigEndian, WriteBytesExt};
use snowflake::ProcessUniqueId;

pub type CursorId = ProcessUniqueId;

word!(WRITE, b"\x85WRITE");
word!(WRITE_END, b"\x80\x85WRITE"); // internal word

word!(READ, b"\x84READ");
word!(READ_END, b"\x80\x84READ"); // internal word

word!(ASSOC, b"\x85ASSOC");
word!(ASSOCQ, b"\x86ASSOC?");
word!(RETR, b"\x84RETR");

word!(CURSOR, b"\x86CURSOR");
word!(QCURSOR_FIRST, b"\x8D?CURSOR/FIRST");
word!(CURSOR_FIRSTQ, b"\x8DCURSOR/FIRST?");
word!(QCURSOR_LAST, b"\x8C?CURSOR/LAST");
word!(CURSOR_LASTQ, b"\x8CCURSOR/LAST?");
word!(QCURSOR_NEXT, b"\x8C?CURSOR/NEXT");
word!(CURSOR_NEXTQ, b"\x8CCURSOR/NEXT?");
word!(QCURSOR_PREV, b"\x8C?CURSOR/PREV");
word!(CURSOR_PREVQ, b"\x8CCURSOR/PREV?");
word!(QCURSOR_SEEK, b"\x8C?CURSOR/SEEK");
word!(CURSOR_SEEKQ, b"\x8CCURSOR/SEEK?");
word!(QCURSOR_CUR, b"\x8B?CURSOR/CUR");
word!(CURSOR_CURQ, b"\x8BCURSOR/CUR?");

word!(COMMIT, b"\x86COMMIT");

use std::collections::BTreeMap;

#[derive(PartialEq)]
enum TxType {
    Read, Write
}

pub struct Handler<'a> {
    db: &'a lmdb::Database<'a>,
    db_env: &'a lmdb::Environment,
    db_write_txn: Option<(EnvId, lmdb::WriteTransaction<'a>)>,
    db_read_txn: Option<(EnvId, lmdb::ReadTransaction<'a>)>,
    cursors: BTreeMap<(EnvId, Vec<u8>), (TxType, lmdb::Cursor<'a, 'a>)>
}


macro_rules! validate_lockout {
    ($env: expr, $name: expr, $pid: expr) => {
        if let Some((pid_, _)) = $name {
            if pid_ != $pid {
                return Err(Error::Reschedule)
            }
        }
    };
}

macro_rules! read_or_write_transaction {
    ($me: expr, $env: expr) => {
        if let Some((_, ref txn)) = $me.db_write_txn {
            txn.deref()
        } else if let Some((_, ref txn)) = $me.db_read_txn {
            txn.deref()
        } else {
            return Err(error_no_transaction!());
        };
    };
}

macro_rules! tx_type {
    ($me: expr, $env: expr) => {
        if let Some((_, _)) = $me.db_write_txn {
            TxType::Write
        } else if let Some((_, _)) = $me.db_read_txn {
            TxType::Read
        } else {
            return Err(error_no_transaction!());
        };
    };
}

const STACK_EMPTY_CLOSURE: &'static [u8] = b"";

macro_rules! qcursor_op {
    ($me: expr, $env: expr, $pid: expr, $op: ident, ($($arg: expr),*)) => {
    {
        validate_lockout!($env, $me.db_read_txn, $pid);
        validate_lockout!($env, $me.db_write_txn, $pid);

        let c = stack_pop!($env);

        let txn = read_or_write_transaction!($me, $env);
        let tuple = ($pid, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let access = txn.access();
        let item = cursor.$op::<[u8], [u8]>(&access, $($arg)*);
        match item {
           Ok((key, val)) => {
                let mut offset = 0;
                let sz = key.len() + val.len() + offset_by_size(key.len()) + offset_by_size(val.len());
                let slice = alloc_slice!(sz, $env);
                write_size_into_slice!(key.len(), &mut slice[offset..]);
                offset += offset_by_size(key.len());
                slice[offset..offset + key.len()].copy_from_slice(key);
                offset += key.len();
                write_size_into_slice!(val.len(), &mut slice[offset..]);
                offset += offset_by_size(val.len());
                slice[offset..offset + val.len()].copy_from_slice(val);
                $env.push(slice);
           }
           // not found
           Err(_) => {
                $env.push(STACK_EMPTY_CLOSURE);
           }
        }
        $me.cursors.insert(tuple, (tx_type!($me, $env), cursor));
        Ok(())
    }
    };
}

macro_rules! cursorq_op {
    ($me: expr, $env: expr, $pid: expr, $op: ident, ($($arg: expr),*)) => {
    {
        validate_lockout!($env, $me.db_read_txn, $pid);
        validate_lockout!($env, $me.db_write_txn, $pid);

        let c = stack_pop!($env);

        let txn = read_or_write_transaction!($me, $env);
        let tuple = ($pid, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let access = txn.access();
        let item = cursor.$op::<[u8], [u8]>(&access, $($arg)*);
        match item {
           Ok((_, _)) => {
                $env.push(STACK_TRUE);
           }
           // not found
           Err(_) => {
                $env.push(STACK_FALSE);
           }
        }
        $me.cursors.insert(tuple, (tx_type!($me, $env), cursor));
        Ok(())
    }
    };
}

impl<'a> Module<'a> for Handler<'a> {

    fn done(&mut self, _: &mut Env, pid: EnvId) {
        let mut is_in_read = false;
        let mut is_in_write = false;

        if let Some((pid_, _)) = self.db_read_txn {
            is_in_read = pid_ == pid;
        }

        if let Some((pid_, _)) = self.db_write_txn {
            is_in_write = pid_ == pid;
        }

        if is_in_read {
            match mem::replace(&mut self.db_read_txn, None) {
                Some((_, txn)) => drop(txn),
                None => ()
            }
        }

        if is_in_write {
            match mem::replace(&mut self.db_write_txn, None) {
                Some((_, txn)) => drop(txn),
                None => ()
            }
        }

    }

    fn handle(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_word!(env, self.handle_write(env, word, pid));
        try_word!(env, self.handle_read(env, word, pid));
        try_word!(env, self.handle_assoc(env, word, pid));
        try_word!(env, self.handle_assocq(env, word, pid));
        try_word!(env, self.handle_retr(env, word, pid));
        try_word!(env, self.handle_commit(env, word, pid));
        try_word!(env, self.handle_cursor(env, word, pid));
        try_word!(env, self.handle_cursor_first(env, word, pid));
        try_word!(env, self.handle_cursor_next(env, word, pid));
        try_word!(env, self.handle_cursor_prev(env, word, pid));
        try_word!(env, self.handle_cursor_last(env, word, pid));
        try_word!(env, self.handle_cursor_seek(env, word, pid));
        try_word!(env, self.handle_cursor_cur(env, word, pid));
        Err(Error::UnknownWord)
    }
}

impl<'a> Handler<'a> {

    pub fn new(db_env: &'a lmdb::Environment, db: &'a lmdb::Database<'a>) -> Self {
        Handler {
            db: db, db_env: db_env,
            db_write_txn: None, db_read_txn: None,
            cursors: BTreeMap::new()
        }
    }


    #[inline]
    pub fn handle_write(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        match word {
            WRITE => {
                let v = stack_pop!(env);

                validate_lockout!(env, self.db_write_txn, pid);
                // prepare transaction
                match lmdb::WriteTransaction::new(self.db_env) {
                    Err(e) => Err(error_database!(e)),
                    Ok(txn) => {
                        self.db_write_txn = Some((pid, txn));
                        env.program.push(WRITE_END);
                        env.program.push(v);
                        Ok(())
                    }
                }
            }
            WRITE_END => {
                validate_lockout!(env, self.db_write_txn, pid);
                self.cursors = mem::replace(&mut self.cursors,
                                            BTreeMap::new()).into_iter()
                    .filter(|t| ((*t).1).0 != TxType::Write).collect();
                self.db_write_txn = None;
                Ok(())
            }
            _ => Err(Error::UnknownWord),
        }
    }

    #[inline]
    pub fn handle_read(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        match word {
            READ => {
                let v = stack_pop!(env);

                validate_lockout!(env, self.db_read_txn, pid);
                // prepare transaction
                match lmdb::ReadTransaction::new(self.db_env) {
                    Err(e) => Err(error_database!(e)),
                    Ok(txn) => {
                        self.db_read_txn = Some((pid, txn));
                        env.program.push(READ_END);
                        env.program.push(v);
                        Ok(())
                    }
                }
            }
            READ_END => {
                validate_lockout!(env, self.db_read_txn, pid);
                self.cursors = mem::replace(&mut self.cursors,
                                            BTreeMap::new()).into_iter()
                    .filter(|t| ((*t).1).0 != TxType::Read).collect();
                self.db_read_txn = None;
                Ok(())
            }
            _ => Err(Error::UnknownWord),
        }
    }

    #[inline]
    pub fn handle_assoc(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == ASSOC {
            validate_lockout!(env, self.db_write_txn, pid);
            if let Some((_, ref txn)) = self.db_write_txn {
                let value = stack_pop!(env);
                let key = stack_pop!(env);

                let mut access = txn.access();

                match access.put(self.db, key, value, lmdb::put::NOOVERWRITE) {
                    Ok(_) => Ok(()),
                    Err(lmdb::Error::Code(code)) if lmdb::error::KEYEXIST == code => Err(error_duplicate_key!(key)),
                    Err(err) => Err(error_database!(err)),
                }
            } else {
                Err(error_no_transaction!())
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_commit(&mut self, _: &Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == COMMIT {
            validate_lockout!(env, self.db_write_txn, pid);
            if let Some((_, txn)) = mem::replace(&mut self.db_write_txn, None) {
                let _ = txn.commit();
                Ok(())
            } else {
                Err(error_no_transaction!())
            }
        } else {
            Err(Error::UnknownWord)
        }
    }


    #[inline]
    pub fn handle_retr(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == RETR {
            validate_lockout!(env, self.db_write_txn, pid);
            validate_lockout!(env, self.db_read_txn, pid);
            let key = stack_pop!(env);

            let txn = read_or_write_transaction!(self, env);
            let access = txn.access();

            return match access.get::<[u8], [u8]>(self.db, key).to_opt() {
                Ok(Some(val)) => {
                    let slice = alloc_and_write!(val, env);
                    env.push(slice);
                    Ok(())
                }
                Ok(None) => Err(error_unknown_key!(key)),
                Err(err) => Err(error_database!(err)),
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_assocq(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == ASSOCQ {
            validate_lockout!(env, self.db_write_txn, pid);
            let key = stack_pop!(env);

            let txn = read_or_write_transaction!(self, env);
            let access = txn.access();

            match access.get::<[u8], [u8]>(self.db, key).to_opt() {
                Ok(Some(_)) => {
                    env.push(STACK_TRUE);
                    Ok(())
                }
                Ok(None) => {
                    env.push(STACK_FALSE);
                    Ok(())
                }
                Err(err) => Err(error_database!(err)),
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    fn cast_away(cursor: lmdb::Cursor) -> lmdb::Cursor<'a,'a> {
        unsafe { ::std::mem::transmute(cursor) }
    }

    #[inline]
    pub fn handle_cursor(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR {
            validate_lockout!(env, self.db_read_txn, pid);
            validate_lockout!(env, self.db_write_txn, pid);

            let txn = read_or_write_transaction!(self, env);
            match txn.cursor(self.db) {
                Ok(cursor) => {
                    let id = CursorId::new();
                    let mut bytes = Vec::new();
                    if cfg!(target_pointer_width = "64") {
                        let _ = bytes.write_u64::<BigEndian>(id.prefix as u64);
                    }
                    if cfg!(target_pointer_width = "32") {
                        let _ = bytes.write_u32::<BigEndian>(id.prefix as u32);
                    }
                    let _ = bytes.write_u64::<BigEndian>(id.offset);
                    self.cursors.insert((pid.clone(), bytes.clone()), (tx_type!(self, env), Handler::cast_away(cursor)));
                    let slice = alloc_and_write!(bytes.as_slice(), env);
                    env.push(slice);
                    Ok(())
                },
                Err(err) => Err(error_database!(err))
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_cursor_first(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_FIRST {
            qcursor_op!(self, env, pid, first, ())
        } else if word == CURSOR_FIRSTQ {
            cursorq_op!(self, env, pid, first, ())
        } else {
            Err(Error::UnknownWord)
        }
    }


    #[inline]
    pub fn handle_cursor_next(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_NEXT {
            qcursor_op!(self, env, pid, next, ())
        } else if word == CURSOR_NEXTQ {
            cursorq_op!(self, env, pid, next, ())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_cursor_prev(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_PREV {
            qcursor_op!(self, env, pid, prev, ())
        } else if word == CURSOR_PREVQ {
            cursorq_op!(self, env, pid, prev, ())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_cursor_last(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_LAST {
            qcursor_op!(self, env, pid, last, ())
        } else if word == CURSOR_LASTQ {
            cursorq_op!(self, env, pid, last, ())
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_cursor_seek(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_SEEK {
            let key = stack_pop!(env);

            qcursor_op!(self, env, pid, seek_range_k, (key))
        } else if word == CURSOR_SEEKQ {
            let key = stack_pop!(env);

            cursorq_op!(self, env, pid, seek_range_k, (key))
        } else {
            Err(Error::UnknownWord)
        }
    }

    #[inline]
    pub fn handle_cursor_cur(&mut self, env: &mut Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == QCURSOR_CUR {
            qcursor_op!(self, env, pid, get_current, ())
        } else if word == CURSOR_CURQ {
            cursorq_op!(self, env, pid, get_current, ())
        } else {
            Err(Error::UnknownWord)
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut, unused_imports)]
mod tests {
    use script::{Env, Scheduler, Error, RequestMessage, ResponseMessage, EnvId, parse, offset_by_size};
    use std::sync::mpsc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use script::binparser;
    use pubsub;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn errors_during_txn() {
        eval!("[[\"Hey\" ASSOC COMMIT] WRITE] TRY [\"Hey\" ASSOC?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
              });
        eval!("[[\"Hey\" ASSOC COMMIT] WRITE] TRY DROP [\"Hey\" \"there\" ASSOC COMMIT] WRITE [\"Hey\" ASSOC?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
              });

    }

    use test::Bencher;

    #[bench]
    fn write_1000_kv_pairs_in_isolated_txns(b: &mut Bencher) {
        bench_eval!("[HLC \"Hello\"] 1000 TIMES [[ASSOC COMMIT] WRITE] 1000 TIMES", b);
    }

}
