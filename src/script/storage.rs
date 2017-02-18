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
use super::{Env, EnvId, PassResult, Error, STACK_TRUE, STACK_FALSE, offset_by_size,
            ERROR_EMPTY_STACK, ERROR_INVALID_VALUE, ERROR_DUPLICATE_KEY, ERROR_NO_TX,
            ERROR_UNKNOWN_KEY, ERROR_DATABASE, ERROR_DECODING};
use core::ops::Deref;
use byteorder::{BigEndian, WriteBytesExt};
use snowflake::ProcessUniqueId;

use script::binparser;

pub type CursorId = ProcessUniqueId;

word!(WRITE, b"\x85WRITE");
word!(WRITE_END, b"\x80\x85WRITE"); // internal word

word!(READ, b"\x84READ");
word!(READ_END, b"\x80\x84READ"); // internal word

word!(ASSOC, b"\x85ASSOC");
word!(ASSOCQ, b"\x86ASSOC?");
word!(RETR, b"\x84RETR");

word!(CURSOR, b"\x86CURSOR");
word!(CURSOR_FIRST, b"\x8CCURSOR/FIRST");
word!(CURSOR_FIRSTP, b"\x8DCURSOR/FIRST?");
word!(CURSOR_LAST, b"\x8BCURSOR/LAST");
word!(CURSOR_LASTP, b"\x8CCURSOR/LAST?");
word!(CURSOR_NEXT, b"\x8BCURSOR/NEXT");
word!(CURSOR_NEXTP, b"\x8CCURSOR/NEXT?");
word!(CURSOR_PREV, b"\x8BCURSOR/PREV");
word!(CURSOR_PREVP, b"\x8CCURSOR/PREV?");
word!(CURSOR_SEEK, b"\x8BCURSOR/SEEK");
word!(CURSOR_SEEKP, b"\x8CCURSOR/SEEK?");
word!(CURSOR_CUR, b"\x8ACURSOR/CUR");
word!(CURSOR_CURP, b"\x8BCURSOR/CUR?");

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
                return Err(($env, Error::Reschedule))
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
            return Err(($env, error_no_transaction!()));
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
            return Err(($env, error_no_transaction!()));
        };
    };
}

const STACK_EMPTY_CLOSURE: &'static [u8] = b"";

macro_rules! cursor_op {
    ($me: expr, $env: expr, $pid: expr, $op: ident, ($($arg: expr),*)) => {
    {
        validate_lockout!($env, $me.db_read_txn, $pid);
        validate_lockout!($env, $me.db_write_txn, $pid);

        let c = stack_pop!($env);

        let txn = read_or_write_transaction!($me, $env);
        let tuple = ($pid, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(($env, error_invalid_value!(c)))
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
                for i in 0..key.len() {
                    slice[offset+i] = key[i];
                }
                offset += key.len();
                write_size_into_slice!(val.len(), &mut slice[offset..]);
                offset += offset_by_size(val.len());
                for i in 0..val.len() {
                    slice[offset+i] = val[i];
                }
                $env.push(slice);
           }
           // not found
           Err(_) => {
                $env.push(STACK_EMPTY_CLOSURE);
           }
        }
        $me.cursors.insert(tuple, (tx_type!($me, $env), cursor));
        Ok(($env, None))
    }
    };
}

macro_rules! cursorp_op {
    ($me: expr, $env: expr, $pid: expr, $op: ident, ($($arg: expr),*)) => {
    {
        validate_lockout!($env, $me.db_read_txn, $pid);
        validate_lockout!($env, $me.db_write_txn, $pid);

        let c = stack_pop!($env);

        let txn = read_or_write_transaction!($me, $env);
        let tuple = ($pid, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(($env, error_invalid_value!(c)))
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
        Ok(($env, None))
    }
    };
}

impl<'a> Handler<'a> {

    pub fn new(db_env: &'a lmdb::Environment, db: &'a lmdb::Database<'a>) -> Self {
        Handler {
            db: db, db_env: db_env,
            db_write_txn: None, db_read_txn: None,
            cursors: BTreeMap::new()
        }
    }

    pub fn cleanup(&mut self, pid: EnvId) {
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

    #[inline]
    pub fn handle_write(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        match word {
            WRITE => {
                let v = stack_pop!(env);
                assert_decodable!(env, v);

                validate_lockout!(env, self.db_write_txn, pid);
                let mut vec = Vec::from(v);
                vec.extend_from_slice(WRITE_END); // transaction end marker
                // prepare transaction
                match lmdb::WriteTransaction::new(self.db_env) {
                    Err(e) => Err((env, error_database!(e))),
                    Ok(txn) => {
                        self.db_write_txn = Some((pid, txn));
                        Ok((env, Some(vec)))
                    }
                }
            }
            WRITE_END => {
                validate_lockout!(env, self.db_write_txn, pid);
                self.cursors = mem::replace(&mut self.cursors,
                                            BTreeMap::new()).into_iter()
                    .filter(|t| ((*t).1).0 != TxType::Write).collect();
                self.db_write_txn = None;
                Ok((env, None))
            }
            _ => Err((env, Error::UnknownWord)),
        }
    }

    #[inline]
    pub fn handle_read(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        match word {
            READ => {
                let v = stack_pop!(env);
                assert_decodable!(env, v);

                validate_lockout!(env, self.db_read_txn, pid);
                let mut vec = Vec::from(v);
                vec.extend_from_slice(READ_END); // transaction end marker
                // prepare transaction
                match lmdb::ReadTransaction::new(self.db_env) {
                    Err(e) => Err((env, error_database!(e))),
                    Ok(txn) => {
                        self.db_read_txn = Some((pid, txn));
                        Ok((env, Some(vec)))
                    }
                }
            }
            READ_END => {
                validate_lockout!(env, self.db_read_txn, pid);
                self.cursors = mem::replace(&mut self.cursors,
                                            BTreeMap::new()).into_iter()
                    .filter(|t| ((*t).1).0 != TxType::Read).collect();
                self.db_read_txn = None;
                Ok((env, None))
            }
            _ => Err((env, Error::UnknownWord)),
        }
    }

    #[inline]
    pub fn handle_assoc(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == ASSOC {
            validate_lockout!(env, self.db_write_txn, pid);
            if let Some((_, ref txn)) = self.db_write_txn {
                let value = stack_pop!(env);
                let key = stack_pop!(env);

                let mut access = txn.access();

                match access.put(self.db, key, value, lmdb::put::NOOVERWRITE) {
                    Ok(_) => Ok((env, None)),
                    Err(lmdb::Error::ValRejected(_)) => Err((env, error_duplicate_key!(key))),
                    Err(err) => Err((env, error_database!(err))),
                }
            } else {
                Err((env, error_no_transaction!()))
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_commit(&mut self, env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == COMMIT {
            validate_lockout!(env, self.db_write_txn, pid);
            if let Some((_, txn)) = mem::replace(&mut self.db_write_txn, None) {
                let _ = txn.commit();
                Ok((env, None))
            } else {
                Err((env, error_no_transaction!()))
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }


    #[inline]
    pub fn handle_retr(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
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
                    Ok((env, None))
                }
                Ok(None) => Err((env, error_unknown_key!(key))),
                Err(err) => Err((env, error_database!(err))),
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_assocq(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == ASSOCQ {
            validate_lockout!(env, self.db_write_txn, pid);
            let key = stack_pop!(env);

            let txn = read_or_write_transaction!(self, env);
            let access = txn.access();

            match access.get::<[u8], [u8]>(self.db, key).to_opt() {
                Ok(Some(_)) => {
                    env.push(STACK_TRUE);
                    Ok((env, None))
                }
                Ok(None) => {
                    env.push(STACK_FALSE);
                    Ok((env, None))
                }
                Err(err) => Err((env, error_database!(err))),
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    fn cast_away(cursor: lmdb::Cursor) -> lmdb::Cursor<'a,'a> {
        unsafe { ::std::mem::transmute(cursor) }
    }

    #[inline]
    pub fn handle_cursor(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
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
                    let slice = alloc_and_write!(bytes, env);
                    env.push(slice);
                    Ok((env, None))
                },
                Err(err) => Err((env, error_database!(err)))
            }
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_first(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_FIRST {
            cursor_op!(self, env, pid, first, ())
        } else if word == CURSOR_FIRSTP {
            cursorp_op!(self, env, pid, first, ())
        }
        else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_next(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_NEXT {
            cursor_op!(self, env, pid, next, ())
        } else if word == CURSOR_NEXTP {
            cursorp_op!(self, env, pid, next, ())
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_prev(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_PREV {
            cursor_op!(self, env, pid, prev, ())
        } else if word == CURSOR_PREVP {
            cursorp_op!(self, env, pid, prev, ())
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_last(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_LAST {
            cursor_op!(self, env, pid, last, ())
        } else if word == CURSOR_LASTP {
            cursorp_op!(self, env, pid, last, ())
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_seek(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_SEEK {
            let key = stack_pop!(env);

            cursor_op!(self, env, pid, seek_range_k, (key))
        } else if word == CURSOR_SEEKP {
            let key = stack_pop!(env);

            cursorp_op!(self, env, pid, seek_range_k, (key))
        } else {
            Err((env, Error::UnknownWord))
        }
    }

    #[inline]
    pub fn handle_cursor_cur(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        if word == CURSOR_CUR {
            cursor_op!(self, env, pid, get_current, ())
        } else if word == CURSOR_CURP {
            cursorp_op!(self, env, pid, get_current, ())
        } else {
            Err((env, Error::UnknownWord))
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut, unused_imports)]
mod tests {
    use script::{Env, VM, Error, RequestMessage, ResponseMessage, EnvId, parse, offset_by_size};
    use std::sync::mpsc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use script::binparser;
    use pubsub;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn write() {
        eval!("[\"Hello\" \"world\" ASSOC COMMIT] WRITE [\"Hello\" RETR] READ",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
              });

        // overwrite
        eval!("[\"Hello\" \"world\" ASSOC \"Hello\" \"world\" ASSOC COMMIT] WRITE",
              env,
              result,
              {
                  assert!(result.is_err());
              });

        // missing key
        eval!("[\"Hello\" \"world\" ASSOC COMMIT] WRITE [\"world\" RETR] READ",
              env,
              result,
              {
                  assert!(result.is_err());
              });

    }

    #[test]
    fn invalid_closures() {
        eval!("1 WRITE", env, result, {
            assert_error!(result, "[\"Decoding error\" [] 5]");
        });
        eval!("2 WRITE", env, result, {
            assert_error!(result, "[\"Decoding error\" [] 5]");
        });
    }

    #[test]
    fn retr() {
        eval!("[\"Hello\" DUP DUP \"world\" ASSOC RETR COMMIT] WRITE SWAP [RETR] READ",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(env.pop(), None);
              });
    }

    #[test]
    fn assocq() {
        eval!("[\"Hello\" DUP \"world\" ASSOC ASSOC? COMMIT] WRITE [\"Hello\" ASSOC? \"Bye\" ASSOC?] READ",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
              });
    }

    #[test]
    fn cursor_no_txn() {
        eval!("CURSOR", env, result, {
             assert!(result.is_err());
        });
    }

    #[test]
    fn invalid_cursor() {
        eval!("[1 CURSOR/FIRST] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[1 CURSOR/LAST] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[1 CURSOR/NEXT] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[1 CURSOR/PREV] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[1 1 CURSOR/SEEK] READ", env, result, {
             assert!(result.is_err());
        });

        eval!("[CURSOR 'c SET] READ [c CURSOR/FIRST] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[CURSOR 'c SET] READ [c CURSOR/LAST] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[CURSOR 'c SET] READ [c CURSOR/NEXT] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[CURSOR 'c SET] READ [c CURSOR/PREV] READ", env, result, {
             assert!(result.is_err());
        });
        eval!("[CURSOR 'c SET] READ [1 c CURSOR/SEEK] READ", env, result, {
             assert!(result.is_err());
        });

    }


    #[test]
    fn cursorp() {
        eval!("[CURSOR CURSOR/FIRST?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
        });
        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR CURSOR/FIRST?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[CURSOR CURSOR/LAST?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
        });
        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR CURSOR/LAST?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST? DROP c CURSOR/NEXT?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
        });

        eval!("[1 2 ASSOC 2 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST? DROP c CURSOR/NEXT?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST? DROP c CURSOR/PREV?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
        });

        eval!("[1 2 ASSOC 2 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST? DROP c CURSOR/LAST?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

        eval!("[CURSOR 1 CURSOR/SEEK?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
        });

        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR 1 CURSOR/SEEK?] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });

    }

    #[test]
    fn cursor_cur() {
        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST? [c CURSOR/CUR UNWRAP] IF] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x02"));
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });
        eval!("[1 2 ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST? [c CURSOR/CUR?] IF] READ", env, result, {
             assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
        });
    }

    #[test]
    fn cursor_first_and_next() {
        eval!("CURSOR/FIRST", env, result, {
             assert!(result.is_err());
        });
        eval!("CURSOR/NEXT", env, result, {
             assert!(result.is_err());
        });
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST UNWRAP c CURSOR/NEXT UNWRAP] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(env.pop(), None);
              });
        // empty db => no first
        eval!("[CURSOR 'c SET c CURSOR/FIRST SOME?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
                  assert_eq!(env.pop(), None);
              });

        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC CURSOR 'c SET c CURSOR/FIRST UNWRAP c CURSOR/NEXT UNWRAP] WRITE",
              env,
              result,
              {
                  assert!(!result.is_err());
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(env.pop(), None);
              });
    }

    #[test]
    fn cursor_last() {
        eval!("CURSOR/LAST", env, result, {
             assert!(result.is_err());
        });
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST UNWRAP] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(env.pop(), None);
              });

        // empty db => no last
        eval!("[CURSOR 'c SET c CURSOR/LAST SOME?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
                  assert_eq!(env.pop(), None);
              });

        // next after last is none
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/LAST DROP c CURSOR/NEXT NONE?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
                  assert_eq!(env.pop(), None);
              });

        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC CURSOR 'c SET c CURSOR/LAST UNWRAP] WRITE",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(env.pop(), None);
              });
    }


    #[test]
    fn cursor_seek() {
        eval!("CURSOR/SEEK", env, result, {
             assert!(result.is_err());
        });
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c \"Hallo\" CURSOR/SEEK UNWRAP] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\""));
                  assert_eq!(env.pop(), None);
              });

        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC CURSOR 'c SET c \"Hallo\" CURSOR/SEEK UNWRAP] WRITE",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\""));
                  assert_eq!(env.pop(), None);
              });
    }

    #[test]
    fn cursor_prev() {
        eval!("CURSOR/PREV", env, result, {
             assert!(result.is_err());
        });
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST UNWRAP c CURSOR/NEXT UNWRAP c CURSOR/PREV UNWRAP] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(env.pop(), None);
              });
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC CURSOR 'c SET c CURSOR/FIRST UNWRAP c CURSOR/NEXT UNWRAP c CURSOR/PREV UNWRAP] WRITE",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Hello\"")); // H > G
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"world\""));
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("\"Goodbye\""));
                  assert_eq!(env.pop(), None);
              });
        // prev before first is none
        eval!("[\"Hello\" \"world\" ASSOC \"Goodbye\" \"world\" ASSOC COMMIT] WRITE [CURSOR 'c SET c CURSOR/FIRST DROP c CURSOR/PREV NONE?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
                  assert_eq!(env.pop(), None);
              });
    }

    #[test]
    fn commit() {
        eval!("[\"Hey\" \"everybody\" ASSOC] WRITE [\"Hey\" RETR] READ",
              env,
              result,
              {
                  assert!(result.is_err());
              });
    }

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
        let path = "pumpkindb-bench-write_1000_kv_pairs_in_isolated_txns";
        fs::create_dir_all(path).expect("can't create directory");
        let env = unsafe {
            lmdb::EnvBuilder::new()
                .expect("can't create env builder")
                .open(path, lmdb::open::Flags::empty(), 0o600)
                .expect("can't open env")
        };

        let db = lmdb::Database::open(&env,
                                      None,
                                      &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
            .expect("can't open database");
        crossbeam::scope(|scope| {
            let publisher = pubsub::Publisher::new();
            let publisher_accessor = publisher.accessor();
            let mut vm = VM::new(&env, &db, publisher_accessor.clone());
            let sender = vm.sender();
            let handle = scope.spawn(move || {
                vm.run();
            });
            let script = parse("[HLC \"Hello\"] 1000 TIMES [[ASSOC COMMIT] WRITE] 1000 TIMES").unwrap();
            let sender_ = sender.clone();
            b.iter(move || {
                let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), script.clone(), callback));
                match receiver.recv() {
                    Ok(ResponseMessage::EnvTerminated(_, _, _)) => (),
                    Ok(ResponseMessage::EnvFailed(_, err, _, _)) => {
                        let _ = sender.send(RequestMessage::Shutdown);
                        panic!("error: {:?}", err)
                    },
                    Err(err) => panic!("recv error: {:?}", err)
                }
            });
            let _ = sender_.send(RequestMessage::Shutdown);
            let _ = handle.join();
        });
        fs::remove_dir_all(path);
    }

}
