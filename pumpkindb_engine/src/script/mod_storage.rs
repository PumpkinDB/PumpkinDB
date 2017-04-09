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
            ERROR_UNKNOWN_KEY, ERROR_DATABASE};
use byteorder::{BigEndian, WriteBytesExt};
use snowflake::ProcessUniqueId;
use std::collections::BTreeMap;
use storage::WriteTransactionContainer;

pub type CursorId = ProcessUniqueId;

const STACK_EMPTY_CLOSURE: &'static [u8] = b"";

instruction!(WRITE, b"\x85WRITE");
instruction!(WRITE_END, b"\x80\x85WRITE"); // internal instruction

instruction!(READ, b"\x84READ");
instruction!(READ_END, b"\x80\x84READ"); // internal instruction

instruction!(ASSOC, b"\x85ASSOC");
instruction!(ASSOCQ, b"\x86ASSOC?");
instruction!(RETR, b"\x84RETR");

instruction!(CURSOR, b"\x86CURSOR");
instruction!(QCURSOR_FIRST, b"\x8D?CURSOR/FIRST");
instruction!(CURSOR_FIRSTQ, b"\x8DCURSOR/FIRST?");
instruction!(QCURSOR_LAST, b"\x8C?CURSOR/LAST");
instruction!(CURSOR_LASTQ, b"\x8CCURSOR/LAST?");
instruction!(QCURSOR_NEXT, b"\x8C?CURSOR/NEXT");
instruction!(CURSOR_NEXTQ, b"\x8CCURSOR/NEXT?");
instruction!(QCURSOR_PREV, b"\x8C?CURSOR/PREV");
instruction!(CURSOR_PREVQ, b"\x8CCURSOR/PREV?");
instruction!(QCURSOR_SEEK, b"\x8C?CURSOR/SEEK");
instruction!(CURSOR_SEEKQ, b"\x8CCURSOR/SEEK?");
instruction!(QCURSOR_CUR, b"\x8B?CURSOR/CUR");
instruction!(CURSOR_CURQ, b"\x8BCURSOR/CUR?");

instruction!(COMMIT, b"\x86COMMIT");

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

#[derive(Debug)]
enum Txn<'a> {
    Read(lmdb::ReadTransaction<'a>),
    Write(WriteTransactionContainer<'a>),
}

impl<'a> Txn<'a> {
    fn access(&self) -> Accessor {
        match self {
            &Txn::Read(ref txn) => Accessor::Const(txn.access()),
            &Txn::Write(ref txn) => Accessor::Write(txn.access()),
        }
    }
    fn cursor(&self, db: &'a lmdb::Database) -> Result<lmdb::Cursor, lmdb::Error> {
        match self {
            &Txn::Read(ref txn) => txn.cursor(db),
            &Txn::Write(ref txn) => txn.cursor(db),
        }
    }
    fn tx_type(&self) -> TxType {
        match self {
            &Txn::Read(_) => TxType::Read,
            &Txn::Write(_) => TxType::Write,
        }
    }
}

pub struct Handler<'a> {
    db: &'a storage::Storage<'a>,
    txns: HashMap<EnvId, Vec<Txn<'a>>>,
    cursors: BTreeMap<(EnvId, Vec<u8>), (TxType, lmdb::Cursor<'a, 'a>)>
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

macro_rules! qcursor_op {
    ($me: expr, $env: expr, $env_id: expr, $op: ident, ($($arg: expr),*)) => {{
        let txn = read_or_write_transaction!($me, &$env_id);
        let c = stack_pop!($env);

        let tuple = ($env_id, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let _ = match txn.access() {
            Accessor::Const(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).map(|item| copy_to_stack($env, item)),
            Accessor::Write(acc) => cursor.$op::<[u8], [u8]>(&acc, $($arg)*).map(|item| copy_to_stack($env, item))
        }.map_err(|_| $env.push(STACK_EMPTY_CLOSURE));
        $me.cursors.insert(tuple, (tx_type!($me, &$env_id), cursor));
        Ok(())
    }};
}

macro_rules! cursorq_op {
    ($me: expr, $env: expr, $env_id: expr, $op: ident, ($($arg: expr),*)) => {{
        let txn = read_or_write_transaction!($me, &$env_id);
        let c = stack_pop!($env);

        let tuple = ($env_id, Vec::from(c));
        let mut cursor = match $me.cursors.remove(&tuple) {
            Some((_, cursor)) => cursor,
            None => return Err(error_invalid_value!(c))
        };
        let contains = match txn.access() {
            Accessor::Const(acc) => {
                let item = cursor.$op::<[u8], [u8]>(&acc, $($arg)*);
                match item {
                    Ok((_, _)) => true,
                    Err(_) => false,
                }
            }
            Accessor::Write(acc) => {
                let item = cursor.$op::<[u8], [u8]>(&acc, $($arg)*);
                match item {
                    Ok((_, _)) => true,
                    Err(_) => false,
                }
            }
        };
        if contains {
            $env.push(STACK_TRUE)
        } else {
            $env.push(STACK_FALSE)
        }
        $me.cursors.insert(tuple, (tx_type!($me, &$env_id), cursor));
        Ok(())
    }};
}

builtins!("mod_storage.builtins");

impl<'a> Dispatcher<'a> for Handler<'a> {
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
        try_instruction!(env, self.handle_builtins(env, instruction, pid));
        try_instruction!(env, self.handle_write(env, instruction, pid));
        try_instruction!(env, self.handle_read(env, instruction, pid));
        try_instruction!(env, self.handle_assoc(env, instruction, pid));
        try_instruction!(env, self.handle_assocq(env, instruction, pid));
        try_instruction!(env, self.handle_retr(env, instruction, pid));
        try_instruction!(env, self.handle_commit(env, instruction, pid));
        try_instruction!(env, self.handle_cursor(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_first(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_next(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_prev(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_last(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_seek(env, instruction, pid));
        try_instruction!(env, self.handle_cursor_cur(env, instruction, pid));
        Err(Error::UnknownInstruction)
    }
}

impl<'a> Handler<'a> {
    pub fn new(db: &'a storage::Storage<'a>) -> Self {
        Handler {
            db: db,
            txns: HashMap::new(),
            cursors: BTreeMap::new()
        }
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
                let v = stack_pop!(env);
                if self.txns.get(&pid).is_some() && self.txns.get(&pid).unwrap().len() > 0 {
                    return Err(error_program!(
                               "Nested WRITEs are not currently allowed".as_bytes(),
                               "".as_bytes(),
                               ERROR_DATABASE));
                }
                match self.db.write() {
                    None => Err(Error::Reschedule),
                    Some(result) =>
                        match result {
                            Err(e) => Err(error_database!(e)),
                            Ok(txn) => {
                                if !self.txns.contains_key(&pid) {
                                    self.txns.insert(pid, Vec::new());
                                }
                                let _ = self.txns.get_mut(&pid).unwrap().push(Txn::Write(txn));
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
                let v = stack_pop!(env);
                match self.db.read() {
                    None => Err(Error::Reschedule),
                    Some(result) =>
                        match result {
                            Err(e) => Err(error_database!(e)),
                            Ok(txn) => {
                                if !self.txns.contains_key(&pid) {
                                    self.txns.insert(pid, Vec::new());
                                }
                                let _ = self.txns.get_mut(&pid).unwrap().push(Txn::Read(txn));
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
    pub fn handle_assoc(&self,
						env: &mut Env<'a>,
						instruction: &'a [u8],
						pid: EnvId)
						-> PassResult<'a> {
        instruction_is!(env, instruction, ASSOC);
        match self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| match txn.tx_type() {
                TxType::Write => Some(txn),
                _ => None
            }) {
            Some(&Txn::Write(ref txn)) => {
                let value = stack_pop!(env);
                let key = stack_pop!(env);

                let mut access = txn.access();

                match access.put(&self.db.db, key, value, lmdb::put::NOOVERWRITE) {
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
        instruction_is!(env, instruction, COMMIT);
        match self.txns.get_mut(&pid)
            .and_then(|vec| vec.pop()) {
            Some(Txn::Write(txn)) => {
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
        instruction_is!(env, instruction, RETR);
        let key = stack_pop!(env);
        self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.access()))
            .map_or_else(|| Err(error_no_transaction!()), |acc| {
                match acc.get::<[u8], [u8]>(&self.db.db, key) {
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
        instruction_is!(env, instruction, ASSOCQ);
        let key = stack_pop!(env);
        self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .and_then(|txn| Some(txn.access()))
            .map_or_else(|| Err(error_no_transaction!()),  |acc| {
                match acc.get::<[u8], [u8]>(&self.db.db, key) {
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
        instruction_is!(env, instruction, CURSOR);
        let cursor = self.txns.get(&pid)
            .and_then(|v| Some(&v[v.len() - 1]))
            .map(|txn| txn.cursor(&self.db.db));
        match cursor {
            Some(cursor) => {
                match cursor {
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
                        self.cursors.insert((pid.clone(), bytes.clone()), (tx_type!(self, pid), Handler::cast_away(cursor)));
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
        if instruction == QCURSOR_FIRST {
            qcursor_op!(self, env, pid, first, ())
        } else if instruction == CURSOR_FIRSTQ {
            cursorq_op!(self, env, pid, first, ())
        } else {
            Err(Error::UnknownInstruction)
        }
    }


    #[inline]
    pub fn handle_cursor_next(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        if instruction == QCURSOR_NEXT {
            qcursor_op!(self, env, pid, next, ())
        } else if instruction == CURSOR_NEXTQ {
            cursorq_op!(self, env, pid, next, ())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_cursor_prev(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        if instruction == QCURSOR_PREV {
            qcursor_op!(self, env, pid, prev, ())
        } else if instruction == CURSOR_PREVQ {
            cursorq_op!(self, env, pid, prev, ())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_cursor_last(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        if instruction == QCURSOR_LAST {
            qcursor_op!(self, env, pid, last, ())
        } else if instruction == CURSOR_LASTQ {
            cursorq_op!(self, env, pid, last, ())
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_cursor_seek(&mut self,
                              env: &mut Env<'a>,
                              instruction: &'a [u8],
                              pid: EnvId)
                              -> PassResult<'a> {
        if instruction == QCURSOR_SEEK {
            let key = stack_pop!(env);

            qcursor_op!(self, env, pid, seek_range_k, (key))
        } else if instruction == CURSOR_SEEKQ {
            let key = stack_pop!(env);

            cursorq_op!(self, env, pid, seek_range_k, (key))
        } else {
            Err(Error::UnknownInstruction)
        }
    }

    #[inline]
    pub fn handle_cursor_cur(&mut self,
                             env: &mut Env<'a>,
                             instruction: &'a [u8],
                             pid: EnvId)
                             -> PassResult<'a> {
        if instruction == QCURSOR_CUR {
            qcursor_op!(self, env, pid, get_current, ())
        } else if instruction == CURSOR_CURQ {
            cursorq_op!(self, env, pid, get_current, ())
        } else {
            Err(Error::UnknownInstruction)
        }
    }
}

fn copy_to_stack(env: &mut Env, (key, val): (&[u8], &[u8])) -> Result<(), Error> {
    let mut offset = 0;
    let sz = key.len() + val.len() + offset_by_size(key.len()) + offset_by_size(val.len());
    let slice = alloc_slice!(sz, env);
    write_size_into_slice!(key.len(), &mut slice[offset..]);
    offset += offset_by_size(key.len());
    slice[offset..offset + key.len()].copy_from_slice(key);
    offset += key.len();
    write_size_into_slice!(val.len(), &mut slice[offset..]);
    offset += offset_by_size(val.len());
    slice[offset..offset + val.len()].copy_from_slice(val);
    env.push(slice);
    Ok(())
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut, unused_imports)]
mod tests {
    use pumpkinscript::{parse, offset_by_size};
    use messaging;
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
        eval!("\"hello\" HLC CONCAT DUP [\"world\" ASSOC [ASSOC?] READ] WRITE 0x00 EQUAL?", env, result, {
            assert_eq!(Vec::from(env.pop().unwrap()), parsed_data!("0x01"));
            assert_eq!(env.pop(), None);
        });
    }

    use test::Bencher;

    #[bench]
    fn write_1000_kv_pairs_in_isolated_txns(b: &mut Bencher) {
        bench_eval!("[HLC \"Hello\"] 1000 TIMES [[ASSOC COMMIT] WRITE] 1000 TIMES",
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
        let timestamp = timestamp::Timestamp::new(None);
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
