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
#[cfg(not(target_os = "windows"))]
use alloc::heap;
#[cfg(not(target_os = "windows"))]
use core::mem::size_of;
use std::sync::Mutex;
use lmdb;

pub struct Storage<'a> {
    pub db: lmdb::Database<'a>,
    pub env: &'a lmdb::Environment,
    pub gwl: Mutex<()>,
}

pub trait GlobalWriteLock {
    fn try_lock(&self) -> bool;
}

impl<'a> GlobalWriteLock for Storage<'a> {
    fn try_lock(&self) -> bool {
        self.gwl.try_lock().is_ok()
    }
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
            gwl: Mutex::new(()),
        }
    }
}

pub fn create_environment(storage_path: String, map_size: Option<i64>) -> lmdb::Environment {
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
                let statp: *mut statvfs =
                    heap::allocate(size_of::<statvfs>(), size_of::<usize>()) as *mut statvfs;
                let mut stat = *statp;
                if statvfs(absolute_path_c.as_ptr(), &mut stat) != 0 {
                    warn!("Can't determine available disk space");
                } else {
                    let size = (stat.f_frsize * stat.f_bavail as u64) as usize;
                    info!("Available disk space is approx. {}Gb, setting database map size to it",
                          size / (1024 * 1024 * 1024));
                    env_builder.set_mapsize(size).expect("can't set map size");
                }
                heap::deallocate(statp as *mut u8, size_of::<statvfs>(), size_of::<usize>());
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
        env_builder.open(storage_path.as_str(), lmdb::open::NOTLS, 0o600)
            .expect("can't open env")
    }
}
