// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]
#![cfg_attr(test, feature(test))]

#![cfg_attr(not(target_os = "windows"), feature(alloc, heap_api))]
#![feature(alloc)]

include!("crates.rs");

extern crate num_cpus;
extern crate log4rs;

pub mod script;
pub mod server;
pub mod logicalstamp;
pub mod pubsub;
pub mod storage;

use std::fs;
use std::thread;
use std::sync::Arc;

use std::sync::Mutex;

lazy_static! {

 static ref ENVIRONMENT: lmdb::Environment = {
    let _ = config::set_default("storage.path", "pumpkin.db");
    let storage_path = config::get_str("storage.path").unwrap().into_owned();
    fs::create_dir_all(storage_path.as_str()).expect("can't create directory");
    let map_size = config::get_int("storage.mapsize");
    storage::create_environment(storage_path, map_size)
 };

 static ref DATABASE: Arc<storage::Storage<'static>> = {
    Arc::new(storage::Storage::new(&ENVIRONMENT))
 };

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

    info!("Starting up");

    let mut senders = Vec::new();

    for i in 0..num_cpus::get() {
        info!("Starting scheduler on core {}.", i);
        let mut scheduler = script::Scheduler::new(
            &DATABASE,
            PUBLISHER.lock().unwrap().clone(),
        );
        let sender = scheduler.sender();
        thread::spawn(move || scheduler.run());
        senders.push(sender)
    }

    server::run(config::get_int("server.port").unwrap(), senders, PUBLISHER.lock().unwrap().clone());

}
