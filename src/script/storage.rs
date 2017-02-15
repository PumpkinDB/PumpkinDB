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
use super::{Env, EnvId, PassResult, Error, STACK_TRUE, STACK_FALSE};
use core::ops::Deref;

word!(WRITE, b"\x85WRITE");
word!(WRITE_END, b"\x80\x85WRITE"); // internal word

word!(READ, b"\x84READ");
word!(READ_END, b"\x80\x84READ"); // internal word

word!(ASSOC, b"\x85ASSOC");
word!(ASSOCQ, b"\x86ASSOC?");
word!(RETR, b"\x84RETR");
word!(COMMIT, b"\x86COMMIT");

pub struct Handler<'a> {
    db: &'a lmdb::Database<'a>,
    db_env: &'a lmdb::Environment,
    db_write_txn: Option<(EnvId, lmdb::WriteTransaction<'a>)>,
    db_read_txn: Option<(EnvId, lmdb::ReadTransaction<'a>)>,
}

impl<'a> Handler<'a> {

    pub fn new(db_env: &'a lmdb::Environment, db: &'a lmdb::Database<'a>) -> Self {
        Handler {
            db: db, db_env: db_env,
            db_write_txn: None, db_read_txn: None
        }
    }

    #[inline]
    pub fn handle_write(&mut self, mut env: Env<'a>, word: &'a [u8], pid: EnvId) -> PassResult<'a> {
        match word {
            WRITE => {
                let v = stack_pop!(env);

                validate_lockout!(env, self.db_write_txn, pid);
                let mut vec = Vec::from(v);
                vec.extend_from_slice(WRITE_END); // transaction end marker
                // prepare transaction
                match lmdb::WriteTransaction::new(self.db_env) {
                    Err(e) => Err((env, Error::DatabaseError(e))),
                    Ok(txn) => {
                        self.db_write_txn = Some((pid, txn));
                        Ok((env, Some(vec)))
                    }
                }
            }
            WRITE_END => {
                validate_lockout!(env, self.db_write_txn, pid);
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

                validate_lockout!(env, self.db_read_txn, pid);
                validate_lockout!(env, self.db_write_txn, pid);
                let mut vec = Vec::from(v);
                vec.extend_from_slice(READ_END); // transaction end marker
                // prepare transaction
                match lmdb::ReadTransaction::new(self.db_env) {
                    Err(e) => Err((env, Error::DatabaseError(e))),
                    Ok(txn) => {
                        self.db_read_txn = Some((pid, txn));
                        Ok((env, Some(vec)))
                    }
                }
            }
            READ_END => {
                validate_lockout!(env, self.db_read_txn, pid);
                validate_lockout!(env, self.db_write_txn, pid);
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
                    Err(lmdb::Error::ValRejected(_)) => Err((env, Error::DuplicateKey)),
                    Err(err) => Err((env, Error::DatabaseError(err))),
                }
            } else {
                Err((env, Error::NoTransaction))
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
                Err((env, Error::NoTransaction))
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
                    let slice0 = env.alloc(val.len());
                    if slice0.is_err() {
                        return Err((env, slice0.unwrap_err()))
                    }
                    let mut slice = slice0.unwrap();
                    for i in 0..val.len() {
                        slice[i] = val[i];
                    }
                    env.push(slice);
                    Ok((env, None))
                }
                Ok(None) => Err((env, Error::UnknownKey)),
                Err(err) => Err((env, Error::DatabaseError(err))),
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
                Err(err) => Err((env, Error::DatabaseError(err))),
            }
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
    fn commit() {
        eval!("[\"Hey\" \"everybody\" ASSOC] WRITE [\"Hey\" RETR] READ",
              env,
              result,
              {
                  assert!(result.is_err());
              });
    }

    use test::Bencher;

    #[bench]
    // This test, even when executed under `cargo test`,
    // is hanging on Travis CI. Unable to figure it out now,
    // disabling it.
    #[cfg(not(feature = "travis"))]
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
            let mut vm = VM::new(&env, &db);
            let sender = vm.sender();
            let handle = scope.spawn(move || {
                vm.run();
            });
            let script = parse("[pair : HLC \"hello\"] SET [pair] 1000 TIMES [[ASSOC COMMIT] WRITE] 1000 TIMES").unwrap();
            let sender_ = sender.clone();
            b.iter(move || {
                let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), script.clone(), callback));
                match receiver.recv() {
                    Ok(ResponseMessage::EnvTerminated(_, _, _)) => (),
                    Ok(ResponseMessage::EnvFailed(_, err, _, _)) => panic!("error: {:?}", err),
                    Err(err) => panic!("recv error: {:?}", err)
                }
            });
            let _ = sender_.send(RequestMessage::Shutdown);
            let _ = handle.join();
        });
        fs::remove_dir_all(path);
    }

}
