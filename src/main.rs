// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]
#![cfg_attr(test, feature(test))]

#![cfg_attr(not(target_os = "windows"), feature(alloc, heap_api))]
#![cfg_attr(target_os = "windows", feature(alloc))]

include!("crates.rs");

extern crate log4rs;
#[macro_use]
extern crate log;

pub mod script;
pub mod server;
pub mod timestamp;
pub mod pubsub;

use std::thread;
use std::sync::Arc;

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

lazy_static! {
 static ref ENV: Arc<lmdb::Environment> = {
     let _ = config::set_default("storage.path", "pumpkin.db");

     let path = config::get_str("storage.path").unwrap().into_owned();
     fs::create_dir_all(path.as_str()).expect("can't create directory");
     unsafe {
            let mut env_builder = lmdb::EnvBuilder::new()
                .expect("can't create env builder");

            // Configure map size
            if !cfg!(target_os = "windows") && config::get_int("storage.mapsize").is_none() {
                #[cfg(not(target_os = "windows"))]
                {
                    let path = path::PathBuf::from(path.as_str());
                    let canonical = fs::canonicalize(&path).unwrap();
                    let absolute_path = canonical.as_path().to_str().unwrap();
                    let absolute_path_c = CString::new(absolute_path).unwrap();
                    let statp: *mut statvfs = heap::allocate(size_of::<statvfs>(), size_of::<usize>()) as *mut statvfs;
                    let mut stat = *statp;
                    if statvfs(absolute_path_c.as_ptr(), &mut stat) != 0 {
                       warn!("Can't determine available disk space");
                    } else {
                       let size = (stat.f_frsize * stat.f_bavail as u64) as usize;
                       info!("Available disk space is approx. {}Gb, setting database map size to it", size / (1024*1024*1024));
                       env_builder.set_mapsize(size).expect("can't set map size");
                    }
                    heap::deallocate(statp as *mut u8, size_of::<statvfs>(), size_of::<usize>());
                }
            } else {
                match config::get_int("storage.mapsize") {
                   Some(mapsize) => {
                       env_builder.set_mapsize(1024 * mapsize as usize).expect("can't set map size");
                   },
                   None => {
                       warn!("No default storage.mapsize set, setting it to 1Gb");
                       env_builder.set_mapsize(1024 * 1024 * 1024).expect("can't set map size");
                   }
                }
            }
            Arc::new(env_builder
                .open(path.as_str(), lmdb::open::Flags::empty(), 0o600)
                .expect("can't open env"))
    }
 };

 static ref DB: Arc<lmdb::Database<'static>> = Arc::new(lmdb::Database::open(ENV.clone(),
                              None,
                              &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                              .expect("can't open database"));

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

    // Initialize logging
    let log_config = config::get_str("logging.config");
    let mut log_configured = false;
    if log_config.is_some() {
        let log_file_path = log_config.unwrap().into_owned();
        if fs::metadata(&log_file_path).is_ok() {
            log4rs::init_file(&log_file_path, Default::default()).unwrap();
            log_configured = true;
        } else {
            println!("{} not found", &log_file_path);
        }
    }

    if !log_configured {
        let appender = log4rs::config::Appender::builder()
            .build("console",
                   Box::new(log4rs::append::console::ConsoleAppender::builder().build()));
        let root = log4rs::config::Root::builder().appender("console").build(log::LogLevelFilter::Info);
        let _ = log4rs::init_config(log4rs::config::Config::builder().appender(appender).build(root).unwrap());
        warn!("No logging configuration specified, switching to console logging");
    }
    //

    info!("Starting up");

    let mut vm = script::VM::new(&ENV, &DB, PUBLISHER.lock().unwrap().clone());
    let sender = vm.sender();

    thread::spawn(move || vm.run());

    server::run(config::get_int("server.port").unwrap(), sender, PUBLISHER.lock().unwrap().clone());

}
