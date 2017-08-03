// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
//! # Storage
//!
//! This module handles all instructions and state related to handling storage
//! capabilities
//!

use lmdb;
use lmdb::traits::{LmdbResultExt, AsLmdbBytes, FromLmdbBytes};
use storage;
use std::mem;
use std::error::Error as StdError;
use std::collections::HashMap;
use super::{Env, EnvId, Dispatcher, PassResult, Error, STACK_TRUE, STACK_FALSE, offset_by_size,
            ERROR_EMPTY_STACK, ERROR_INVALID_VALUE, ERROR_DUPLICATE_KEY, ERROR_NO_TX,
            ERROR_UNKNOWN_KEY, ERROR_DATABASE, ERROR_NO_VALUE, TryInstruction};
use snowflake::ProcessUniqueId;
use std::collections::BTreeMap;
use storage::WriteTransactionContainer;
use num_bigint::BigUint;
use num_traits::FromPrimitive;

pub type CursorId = ProcessUniqueId;

instruction!(TXID, b"\x84TXID");
instruction!(WRITE, b"\x85WRITE");
instruction!(WRITE_END, b"\x80\x85WRITE"); // internal instruction

instruction!(READ, b"\x84READ");
instruction!(READ_END, b"\x80\x84READ"); // internal instruction

instruction!(ASSOC, b"\x85ASSOC");
instruction!(ASSOCQ, b"\x86ASSOC?");
instruction!(RETR, b"\x84RETR");

instruction!(CURSOR, b"\x86CURSOR");
instruction!(CURSOR_FIRST, b"\x8CCURSOR/FIRST");
instruction!(CURSOR_LAST, b"\x8BCURSOR/LAST");
instruction!(CURSOR_NEXT, b"\x8BCURSOR/NEXT");
instruction!(CURSOR_PREV, b"\x8BCURSOR/PREV");
instruction!(CURSOR_SEEK, b"\x8BCURSOR/SEEK");
instruction!(CURSOR_POSITIONEDQ, b"\x92CURSOR/POSITIONED?");
instruction!(CURSOR_KEY, b"\x8ACURSOR/KEY");
instruction!(CURSOR_VAL, b"\x8ACURSOR/VAL");

instruction!(COMMIT, b"\x86COMMIT");

instruction!(MAXKEYSIZE, b"\x92$SYSTEM/MAXKEYSIZE");

#[derive(PartialEq, Debug)]
enum TxType {
    Read,
    Write,
}

enum Accessor<'a> {
    Const(lmdb::ConstAccessor<'a>),
    Write(lmdb::WriteAccessor<'a>),
}

impl<'a> Accessor<'a> {
    fn get<K: AsLmdbBytes + ?Sized, V: FromLmdbBytes + ?Sized>(&self, db: &lmdb::Database, key: &K)
        -> Result<Option<&V>, lmdb::Error> {
        match self {
            &Accessor::Write(ref access) => {
                access.get::<K, V>(db, key)
            },
            &Accessor::Const(ref access) => {
                access.get::<K, V>(db, key)
            }
        }.to_opt()
    }
}

type TxnId<'a> = &'a [u8];

#[derive(Debug)]
enum Txn<'a> {
    Read(lmdb::ReadTransaction<'a>, TxnId<'a>),
    Write(WriteTransactionContainer<'a>, TxnId<'a>),
}

impl<'a> Txn<'a> {
    fn access(&self) -> Accessor {
        match self {
            &Txn::Read(ref txn, _) => Accessor::Const(txn.access()),
            &Txn::Write(ref txn, _) => Accessor::Write(txn.access()),
        }
    }
    fn cursor(&self, db: &'a lmdb::Database) -> Result<lmdb::Cursor, lmdb::Error> {
        match self {
            &Txn::Read(ref txn, _) => txn.cursor(db),
            &Txn::Write(ref txn, _) => txn.cursor(db),
        }
    }
    fn tx_type(&self) -> TxType {
        match self {
            &Txn::Read(_, _) => TxType::Read,
            &Txn::Write(_, _) => TxType::Write,
        }
    }
    fn id(&self) -> TxnId<'a> {
        match self {
            &Txn::Read(_, txid) => txid,
            &Txn::Write(_, txid) => txid,
        }
    }
}

use std::sync::Arc;
use super::super::timestamp;
use super::super::nvmem::NonVolatileMemory;

pub struct Handler<'a, T, N>
    where T : AsRef<storage::Storage<'a>> + 'a,
          N : NonVolatileMemory {
    db: T,
    txns: HashMap<EnvId, Vec<Txn<'a>>>,
    cursors: BTreeMap<(EnvId, Vec<u8>), (TxType, lmdb::Cursor<'a, 'a>)>,
    maxkeysize: Vec<u8>,
    timestamp: Arc<timestamp::Timestamp<N>>,
}

macro_rules! read_or_write_transaction {
    ($me: expr, $env_id: expr) => {
        match $me.txns.get(&$env_id)
            .and_then(|v| Some(&v[v.len() - 1])) {
            None => return Err(error_no_transaction!()),
            Some(txn) => txn
        }
    };
}

macro_rules! tx_type {
    ($me: expr, $env_id: expr) => {{
        let txn_type = $me.txns.get(&$env_id)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.tx_type()));
        if txn_type.is_none() {
            return Err(error_no_transaction!())
        }
        txn_type.unwrap()
    }};
}

macro_rules! cursor_op {
    ($me: expr, $env: expr, $env_id: expr, $op: ident, ($($arg: expr),*)) => {{
        let txn = read_or_write_transaction!($me, &$env_id);
        let c = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let tuple = ($env_id, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let result = match txn.access() {
            Accessor::Const(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).is_ok(),
            Accessor::Write(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).is_ok()
        };
        $me.cursors.insert(tuple, (tx_type!($me, &$env_id), cursor));
        if result {
          $env.push(STACK_TRUE);
        } else {
          $env.push(STACK_FALSE);
        }
    }};
}

macro_rules! cursor_map_op {
    ($me: expr, $env: expr, $env_id: expr, $op: ident, ($($arg: expr),*), $map: expr, $orelse: expr) => {{
        let txn = read_or_write_transaction!($me, &$env_id);
        let c = $env.pop().ok_or_else(|| error_empty_stack!())?;

        let tuple = ($env_id, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let result = match txn.access() {
            Accessor::Const(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).map_err($orelse).and_then($map),
            Accessor::Write(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).map_err($orelse).and_then($map)
        };
        $me.cursors.insert(tuple, (tx_type!($me, &$env_id), cursor));
        result
    }};
}

builtins!("mod_storage.psc");

impl<'a, T, N> Dispatcher<'a> for Handler<'a, T, N>
    where T : AsRef<storage::Storage<'a>> + 'a,
          N : NonVolatileMemory {
    fn done(&mut self, _: &mut Env, pid: EnvId) {
        self.txns.get_mut(&pid)
            .and_then(|vec| {
                while vec.len() > 0 {
                    let txn = vec.pop();
                    drop(txn)
                }
                Some(())
            });
    }

    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_builtins(env, instruction, pid)
        .if_unhandled_try(|| self.handle_write(env, instruction, pid))
        .if_unhandled_try(|| self.handle_read(env, instruction, pid))
        .if_unhandled_try(|| self.handle_txid(env, instruction, pid))
        .if_unhandled_try(|| self.handle_assoc(env, instruction, pid))
        .if_unhandled_try(|| self.handle_assocq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_retr(env, instruction, pid))
        .if_unhandled_try(|| self.handle_commit(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_first(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_next(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_prev(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_last(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_seek(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_positionedq(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_key(env, instruction, pid))
        .if_unhandled_try(|| self.handle_cursor_val(env, instruction, pid))
        .if_unhandled_try(|| self.handle_maxkeysize(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a, T, N> Handler<'a, T, N>
    where T : AsRef<storage::Storage<'a>> + 'a,
          N : NonVolatileMemory {
    pub fn new(db: T, timestamp: Arc<timestamp::Timestamp<N>>) -> Self {
        let maxkeysize = BigUint::from_u32(db.as_ref().env.maxkeysize()).unwrap().to_bytes_be();
        Handler {
            db: db,
            txns: HashMap::new(),
            cursors: BTreeMap::new(),
            maxkeysize: maxkeysize,
            timestamp,
        }
    }

    fn new_txid(&self, env: &mut Env<'a>) -> Result<TxnId<'a>, super::Error> {
        let now = self.timestamp.hlc();
        let slice = env.alloc(16);
        if slice.is_err() {
            return Err(slice.unwrap_err());
        }
        let mut slice = slice.unwrap();
        let _ = now.write_bytes(&mut slice[0..]).unwrap();
        Ok(slice)
    }

    handle_builtins!();

    #[inline]
    pub fn handle_write(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        pid: EnvId)
                        -> PassResult<'a> {
        match instruction {
            WRITE => {
                let v = env.pop().ok_or_else(|| error_empty_stack!())?;
                if self.txns.get(&pid).is_some() && self.txns.get(&pid).unwrap().len() > 0 {
                    return Err(error_program!(
                               "Nested WRITEs are not currently allowed".as_bytes(),
                               "".as_bytes(),
                               ERROR_DATABASE));
                }
                match self.db.as_ref().write() {
                    None => Err(Error::Reschedule),
                    Some(result) =>
                        match result {
                            Err(e) => Err(error_database!(e)),
                            Ok(txn) => {
                                let txid = self.new_txid(env).unwrap();
                                if !self.txns.contains_key(&pid) {
                                    self.txns.insert(pid, Vec::new());
                                }
                                let _ = self.txns.get_mut(&pid).unwrap().push(Txn::Write(txn, txid));
                                env.program.push(WRITE_END);
                                env.program.push(v);
                                Ok(())
                            }
                        }
                }
            }
            WRITE_END => {
                match self.txns.get_mut(&pid).unwrap().pop() {
                    Some(_) => {
                        self.cursors = mem::replace(&mut self.cursors,
                                                    BTreeMap::new()).into_iter()
                            .filter(|t| ((*t).1).0 != TxType::Read).collect();
                    },
                    _ => {}
                };
                Ok(())
            }
            _ => Err(Error::UnknownInstruction),
        }
    }

    #[inline]
    pub fn handle_read(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       pid: EnvId)
                       -> PassResult<'a> {
        match instruction {
            READ => {
                let v = env.pop().ok_or_else(|| error_empty_stack!())?;
                match self.db.as_ref().read() {
                    None => Err(Error::Reschedule),
                    Some(result) =>
                        match result {
                            Err(e) => Err(error_database!(e)),
                            Ok(txn) => {
                                let txid = self.new_txid(env).unwrap();
                                if !self.txns.contains_key(&pid) {
                                    self.txns.insert(pid, Vec::new());
                                }
                                let _ = self.txns.get_mut(&pid).unwrap().push(Txn::Read(txn, txid));
                                env.program.push(READ_END);
                                env.program.push(v);
                                Ok(())
                            }
                    }
                }
            }
            READ_END => {
                match self.txns.get_mut(&pid).unwrap().pop() {
                    Some(_) => {
                        self.cursors = mem::replace(&mut self.cursors,
                                                    BTreeMap::new()).into_iter()
                            .filter(|t| ((*t).1).0 != TxType::Read).collect();
                    },
                    _ => {}
                };
                Ok(())
            }
            _ => Err(Error::UnknownInstruction),
        }
    }


    #[inline]
    pub fn handle_txid(&self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       pid: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TXID);
        self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.id()))
            .map_or_else(|| Err(error_no_transaction!()),  |txid| {
                env.push(txid);
                Ok(())
            })
    }

    #[inline]
    pub fn handle_assoc(&self,
						env: &mut Env<'a>,
						instruction: &'a [u8],
						pid: EnvId)
						-> PassResult<'a> {
        return_unless_instructions_equal!(instruction, ASSOC);
        match self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| match txn.tx_type() {
                TxType::Write => Some(txn),
                _ => None
            }) {
            Some(&Txn::Write(ref txn, _)) => {
                let value = env.pop().ok_or_else(|| error_empty_stack!())?;
                let key = env.pop().ok_or_else(|| error_empty_stack!())?;

                let mut access = txn.access();

                match access.put(&self.db.as_ref().db, key, value, lmdb::put::NOOVERWRITE) {
                    Ok(_) => Ok(()),
                    Err(lmdb::Error::Code(code)) if lmdb::error::KEYEXIST == code => Err(error_duplicate_key!(key)),
                    Err(err) => Err(error_database!(err)),
                }
            },
            _ => Err(error_no_transaction!())
        }
    }

    #[inline]
    pub fn handle_commit(&mut self,
						 _: &Env<'a>,
						 instruction: &'a [u8],
						 pid: EnvId)
						 -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, COMMIT);
        match self.txns.get_mut(&pid)
            .and_then(|vec| vec.pop()) {
            Some(Txn::Write(txn, _)) => {
                match txn.commit() {
                    Ok(_) => Ok(()),
                    Err(reason) => Err(error_database!(reason))
                }
            },
            Some(txn) => {
                let _ = self.txns.get_mut(&pid).unwrap().push(txn);
                Err(error_no_transaction!())
            },
            None => Err(error_no_transaction!())
        }
    }


    #[inline]
    pub fn handle_retr(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       pid: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, RETR);
        let key = env.pop().ok_or_else(|| error_empty_stack!())?;
        self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.access()))
            .map_or_else(|| Err(error_no_transaction!()), |acc| {
                match acc.get::<[u8], [u8]>(&self.db.as_ref().db, key) {
                    Ok(Some(val)) => {
                        let slice = alloc_and_write!(val, env);
                        env.push(slice);
                        Ok(())
                    },
                    Ok(None) => Err(error_unknown_key!(key)),
                    Err(err) => Err(error_database!(err)),
                }
            })
    }

    #[inline]
    pub fn handle_assocq(&mut self,
                         env: &mut Env<'a>,
                         instruction: &'a [u8],
                         pid: EnvId)
                         -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, ASSOCQ);
        let key = env.pop().ok_or_else(|| error_empty_stack!())?;
        self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.access()))
            .map_or_else(|| Err(error_no_transaction!()),  |acc| {
                match acc.get::<[u8], [u8]>(&self.db.as_ref().db, key) {
                    Ok(Some(_)) => {
                        env.push(STACK_TRUE);
                        Ok(())
                    },
                    Ok(None) => {
                        env.push(STACK_FALSE);
                        Ok(())
                    }
                    Err(err) => Err(error_database!(err)),
                }
            })
    }

    fn cast_away(cursor: lmdb::Cursor) -> lmdb::Cursor<'a, 'a> {
        unsafe { ::std::mem::transmute(cursor) }
    }

    #[inline]
    pub fn handle_cursor(&mut self,
						 env: &mut Env<'a>,
						 instruction: &'a [u8],
						 pid: EnvId)
						 -> PassResult<'a> {
        use serde_cbor;
        return_unless_instructions_equal!(instruction, CURSOR);
        let db = self.db.as_ref();
        let cursor = self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .map(|txn| txn.cursor(&db.db));
        match cursor {
            Some(cursor) => {
                match cursor {
                    Ok(cursor) => {
                        let id = CursorId::new();
                        let bytes = serde_cbor::to_vec(&id).unwrap();
                        self.cursors.insert((pid.clone(), bytes.clone()),
                                            (tx_type!(self, pid),
                                             Handler::<T, N>::cast_away(cursor)));
                        let slice = alloc_and_write!(bytes.as_slice(), env);
                        env.push(slice);
                        Ok(())
                    },
                    Err(err) => Err(error_database!(err))
                }
            },
            None => Err(error_no_transaction!()),
        }
    }

    #[inline]
    pub fn handle_cursor_first(&mut self,
                               env: &mut Env<'a>,
                               instruction: &'a [u8],
                               pid: EnvId)
                               -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_FIRST);
        cursor_op!(self, env, pid, first, ());
        Ok(())
    }


    #[inline]
    pub fn handle_cursor_next(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_NEXT);
        cursor_op!(self, env, pid, next, ());
        Ok(())
    }

    #[inline]
    pub fn handle_cursor_prev(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_PREV);
        cursor_op!(self, env, pid, prev, ());
        Ok(())
    }

    #[inline]
    pub fn handle_cursor_last(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_LAST);
        cursor_op!(self, env, pid, last, ());
        Ok(())
    }

    #[inline]
    pub fn handle_cursor_seek(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_SEEK);
        let key = env.pop().ok_or_else(|| error_empty_stack!())?;
        cursor_op!(self, env, pid, seek_range_k, (key));
        Ok(())
    }

    #[inline]
    pub fn handle_cursor_positionedq(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_POSITIONEDQ);
        let result = match cursor_map_op!(self, env, pid, get_current, (), |_| Ok(true), |_| false) {
            Ok(true) => STACK_TRUE,
            Err(false) => STACK_FALSE,
            Err(true) | Ok(false) => unreachable!(),
        };
        env.push(result);
        Ok(())
    }

    #[inline]
    pub fn handle_cursor_key(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             pid: EnvId)
                             -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_KEY);
        cursor_map_op!(self, env, pid, get_current, (),
           |(key, _) | {
              let slice = alloc_slice!(key.len(), env);
              slice.copy_from_slice(key);
              env.push(slice);
              Ok(())
        }, |_| error_no_value!())
    }

    #[inline]
    pub fn handle_cursor_val(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             pid: EnvId)
                             -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, CURSOR_VAL);
        cursor_map_op!(self, env, pid, get_current, (),
           |(_, val) | {
              let slice = alloc_slice!(val.len(), env);
              slice.copy_from_slice(val);
              env.push(slice);
              Ok(())
        }, |_| error_no_value!())
    }

    #[inline]
    pub fn handle_maxkeysize(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             _: EnvId)
                             -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, MAXKEYSIZE);
        let slice = alloc_and_write!(self.maxkeysize.as_slice(), env);
        env.push(slice);
        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut, unused_imports)]
mod tests {
    use pumpkinscript::{parse, offset_by_size};
    use messaging;
    use nvmem::{MmapedFile, MmapedRegion, NonVolatileMemory};
    use script::{Env, Scheduler, Error, RequestMessage, ResponseMessage, EnvId, dispatcher};

    use byteorder::WriteBytesExt;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use script::binparser;
    use storage;
    use timestamp;
    use rand::Rng;

    const _EMPTY: &'static [u8] = b"";

    #[test]
    fn errors_during_txn() {
        eval!("[[\"Hey\" ASSOC COMMIT] WRITE] TRY [\"Hey\" ASSOC?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x00"));
              });
        eval!("[[\"Hey\" ASSOC COMMIT] WRITE] TRY DROP [\"Hey\" \"there\" ASSOC COMMIT] WRITE \
               [\"Hey\" ASSOC?] READ",
              env,
              result,
              {
                  assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
              });

    }

    #[test]
    fn txn_order() {
        eval!("[\"hello\" HLC CONCAT DUP \"world\" ASSOC [ASSOC?] READ] WRITE 0x00 EQUAL?", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });
    }

    use test::Bencher;

    #[bench]
    fn write_1000_kv_pairs_in_isolated_txns(b: &mut Bencher) {
        bench_eval!("[\"Hello\" >Q HLC >Q] 1000 TIMES [[Q> Q> ASSOC COMMIT] WRITE] 1000 TIMES",
                    b);
    }

    #[bench]
    fn write_1000_kv_pairs_in_isolated_txns_baseline(b: &mut Bencher) {
        let dir = TempDir::new("pumpkindb").unwrap();
        let path = dir.path().to_str().unwrap();
        fs::create_dir_all(path).expect("can't create directory");
        let env = unsafe {
            let mut builder = lmdb::EnvBuilder::new().expect("can't create env builder");
            builder.set_mapsize(1024 * 1024 * 1024).expect("can't set mapsize");
            builder.open(path, lmdb::open::NOTLS, 0o600).expect("can't open env")
        };
        let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
        let region = nvmem.claim(20).unwrap();
        let timestamp = Arc::new(timestamp::Timestamp::new(region));
        let db = storage::Storage::new(&env);
        b.iter(move || {
            let mut timestamps = Vec::new();
            for i in 0..1000 {
                timestamps.push(timestamp.hlc());
            }
            for ts in timestamps {
                let txn = lmdb::WriteTransaction::new(db.env).unwrap();
                {
                    let mut access = txn.access();
                    let mut key: Vec<u8> = Vec::new();

                    ts.write_bytes(&mut key);

                    let _ = access.put(&db.db,
                             key.as_slice(),
                             "Hello".as_bytes(),
                             lmdb::put::NOOVERWRITE)
                        .unwrap();
                }
                let _ = txn.commit().unwrap();
            }
        });
    }

}
