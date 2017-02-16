// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]
#![cfg_attr(test, feature(test))]

#![feature(alloc, heap_api)]

include!("crates.rs");

pub mod script;
pub mod server;
pub mod timestamp;
pub mod pubsub;

use std::thread;

use std::fs;
use std::path;

use std::ffi::CString;
use libc::statvfs;

use alloc::heap;
use core::mem::size_of;

use std::sync::Mutex;

lazy_static! {
 static ref ENV: lmdb::Environment = {
     let _ = config::set_default("storage.path", "pumpkin.db");

     let path = config::get_str("storage.path").unwrap().into_owned();
     fs::create_dir_all(path.as_str()).expect("can't create directory");
     unsafe {
            let mut env_builder = lmdb::EnvBuilder::new()
                .expect("can't create env builder");

            // Configure map size
            if !cfg!(target_os = "windows") {
                let path = path::PathBuf::from(path.as_str());
                let canonical = fs::canonicalize(&path).unwrap();
                let absolute_path = canonical.as_path().to_str().unwrap();
                let absolute_path_c = CString::new(absolute_path).unwrap();
                let statp: *mut statvfs = heap::allocate(size_of::<statvfs>(), size_of::<usize>()) as *mut statvfs;
                let mut stat = *statp;
                if statvfs(absolute_path_c.as_ptr(), &mut stat) != 0 {
                   println!("Can't determine available disk space");
                } else {
                   let size = (stat.f_frsize * stat.f_bavail as u64) as usize;
                   println!("Available disk space is approx. {}Gb, setting database map size to it", size / (1024*1024*1024));
                   env_builder.set_mapsize(size).expect("can't set map size");
                }
                heap::deallocate(statp as *mut u8, size_of::<statvfs>(), size_of::<usize>());
            }
            env_builder
                .open(path.as_str(), lmdb::open::Flags::empty(), 0o600)
                .expect("can't open env")
    }
 };

 static ref DB: lmdb::Database<'static> = lmdb::Database::open(&ENV,
                              None,
                              &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                              .expect("can't open database");

 static ref PUBLISHER: Mutex<pubsub::PublisherAccessor<Vec<u8>>> = {
     let mut publisher = pubsub::Publisher::new();
     let publisher_accessor = publisher.accessor();
     let _ = thread::spawn(move || publisher.run());
     Mutex::new(publisher_accessor)
 };

}

fn main() {
    let _ = config::merge(config::Environment::new("pumpkindb"));
    let _ = config::merge(config::File::new("pumpkindb.toml", config::FileFormat::Toml));

    let _ = config::set_default("server.port", 9981);

    let mut vm = script::VM::new(&ENV, &DB, PUBLISHER.lock().unwrap().clone());
    let sender = vm.sender();

    thread::spawn(move || vm.run());

    server::run(config::get_int("server.port").unwrap(), sender, PUBLISHER.lock().unwrap().clone());

}
