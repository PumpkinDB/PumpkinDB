// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use std::fs;
#[cfg(not(target_os = "windows"))]
use std::path;
#[cfg(not(target_os = "windows"))]
use std::ffi::CString;
#[cfg(not(target_os = "windows"))]
use libc::statvfs;
use lmdb;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct WriteTransactionContainer<'a>(Option<lmdb::WriteTransaction<'a>>, Arc<AtomicBool>);

use core::ops::Deref;

impl<'a> WriteTransactionContainer<'a> {
    pub fn commit(mut self) -> Result<(), lmdb::Error> {
        let commit = ::std::mem::replace(&mut self.0, None).unwrap().commit();
        self.1.compare_and_swap(true, false, Ordering::SeqCst);
        commit
    }
}

impl<'a> Deref for WriteTransactionContainer<'a> {
    type Target = lmdb::WriteTransaction<'a>;

    fn deref(&self) -> &lmdb::WriteTransaction<'a> {
        match self.0 {
            Some(ref txn) => txn,
            None => panic!("no transaction available")
        }
    }
}

impl<'a> Drop for WriteTransactionContainer<'a> {
    fn drop(&mut self) {
        self.1.compare_and_swap(true, false, Ordering::SeqCst);
    }
}

impl<'a> ::std::fmt::Debug for WriteTransactionContainer<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Storage<'a> {
    pub db: lmdb::Database<'a>,
    pub env: &'a lmdb::Environment,
    pub write: Arc<AtomicBool>,
}

impl<'a> Storage<'a> {
    pub fn new(env: &'a lmdb::Environment) -> Storage<'a> {
        if !env.flags().unwrap().contains(lmdb::open::NOTLS) {
            panic!("env should have NOTLS enabled");
        }
        Storage {
            env: env,
            db: lmdb::Database::open(env, None, &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                .expect("can't open database"),
            write: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn write(&self) -> Option<Result<WriteTransactionContainer<'a>, lmdb::Error>> {
        match self.write.compare_and_swap(false, true, Ordering::SeqCst) {
            false => {
                match lmdb::WriteTransaction::new(self.env) {
                    Ok(txn) => Some(Ok(WriteTransactionContainer(Some(txn), self.write.clone()))),
                    Err(err) => Some(Err(err))
                }
            },
            true => None
        }
    }

    pub fn read(&self) -> Option<Result<lmdb::ReadTransaction<'a>, lmdb::Error>> {
        match lmdb::ReadTransaction::new(self.env) {
            Ok(txn) => Some(Ok(txn)),
            // MDB_READERS_FULL
            Err(lmdb::Error::Code(-30790)) => None,
            Err(err) => Some(Err(err))
        }
    }
}

pub fn create_environment(storage_path: String, map_size: Option<i64>, maxreaders: Option<u32>) -> lmdb::Environment {
    unsafe {
        let mut env_builder = lmdb::EnvBuilder::new().expect("can't create env builder");

        // Configure map size
        if !cfg!(target_os = "windows") && map_size.is_none() {
            #[cfg(not(target_os = "windows"))]
            {
                let path = path::PathBuf::from(storage_path.as_str());
                let canonical = fs::canonicalize(&path).unwrap();
                let absolute_path = canonical.as_path().to_str().unwrap();
                let absolute_path_c = CString::new(absolute_path).unwrap();
                let mut stat: statvfs = ::std::mem::zeroed();
                if statvfs(absolute_path_c.as_ptr(), &mut stat) != 0 {
                    warn!("Can't determine available disk space");
                } else {
                    let size = (stat.f_frsize * stat.f_bavail as u64) as usize;
                    info!("Available disk space is approx. {}Gb, setting database map size to it",
                          size / (1024 * 1024 * 1024));
                    env_builder.set_mapsize(size).expect("can't set map size");
                }
            }
        } else {
            match map_size {
                Some(mapsize) => {
                    env_builder.set_mapsize(1024 * mapsize as usize).expect("can't set map size");
                }
                None => {
                    warn!("No default storage.mapsize set, setting it to 1Gb");
                    env_builder.set_mapsize(1024 * 1024 * 1024).expect("can't set map size");
                }
            }
        }
        if let Some(max) = maxreaders {
            let _ = env_builder.set_maxreaders(max);
        }

        env_builder.open(storage_path.as_str(), lmdb::open::NOTLS, 0o600)
            .expect("can't open env")
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {
    use std::fs;
    use tempdir::TempDir;
    use lmdb;

    use std::sync::Arc;

    use storage;

    #[test]
    pub fn read_limit() {
        let dir = TempDir::new("pumpkindb").unwrap();
        let path = dir.path().to_str().unwrap();
        fs::create_dir_all(path).expect("can't create directory");
        let env = unsafe {
            lmdb::EnvBuilder::new()
                .expect("can't create env builder")
                .open(path, lmdb::open::NOTLS, 0o600)
                .expect("can't open env")
        };
        let maxreaders = env.maxreaders().unwrap();

        let db = storage::Storage::new(&env);

        let mut readers = vec![];

        // While we are exhausting maxreaders,
        // we should be able to get a read transaction
        for _ in 0..maxreaders {
            let r = db.read();
            assert!(r.is_some());
            readers.push(r);
        }

        // but when the limit is exhausted,
        // no read transaction should be available
        assert!(db.read().is_none());

        readers.pop();

        // after we've popped one transaction,
        // we should be able to get it
        let r = db.read();
        assert!(r.is_some());
        assert!(db.read().is_none());
    }

    use std::sync::mpsc;
    use crossbeam;

    #[test]
    pub fn write_limit() {
        let dir = TempDir::new("pumpkindb").unwrap();
        let path = dir.path().to_str().unwrap();
        fs::create_dir_all(path).expect("can't create directory");
        let env = unsafe {
            lmdb::EnvBuilder::new()
                .expect("can't create env builder")
                .open(path, lmdb::open::NOTLS, 0o600)
                .expect("can't open env")
        };

        let storage = Arc::new(storage::Storage::new(&env));

        crossbeam::scope(|scope| {

            let db = &(storage.clone());

            let w = db.write();
            assert!(w.is_some());
            assert!(db.write().is_none());
            drop(w);
            // after dropping WriteTransactionContainer, write transactions
            // can be initiated again
            assert!(db.write().is_some());
            drop(db);

            // thread test
            let (sender_c1, receiver_c1) = mpsc::channel();
            let (sender_c2, receiver_c2) = mpsc::channel();

            let db_1 = storage.clone();
            let (sender_1, receiver_1) = mpsc::channel();
            let thread1 = scope.spawn(move || {
                let storage = &db_1;
                let w = storage.write();
                let result = w.is_some();
                let _ = sender_c1.send(result);
                receiver_1.recv();
                drop(w);
                let _ = sender_c1.send(true);
            });

            // wait until thread 1 got the write transaction
            assert!(receiver_c1.recv().unwrap());

            let db_2 = storage.clone();
            let (sender_2, receiver_2) = mpsc::channel();
            let thread2 = scope.spawn(move || {
                let storage = &db_2;
                let w = storage.write();
                let result = w.is_some();
                let _ = sender_c2.send(result);
                receiver_2.recv();
                let w = storage.write();
                let result = w.is_some();
                let _ = sender_c2.send(result);
                receiver_2.recv();
            });

            // wait until thread 2 got rejected for a write transaction
            assert!(!receiver_c2.recv().unwrap());
            // drop the 1st thread
            sender_1.send(());
            assert!(receiver_c1.recv().unwrap());
            // now the second thread should be able to receive a write transaction
            sender_2.send(());
            assert!(receiver_c2.recv().unwrap());

            // drop the 2nd thread
            sender_2.send(());

            // terminate threads
            thread1.join();
            thread2.join();
        });
    }

}