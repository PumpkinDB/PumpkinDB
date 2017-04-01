// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]
#![cfg_attr(test, feature(test))]

extern crate mio;
extern crate memmap;
extern crate byteorder;
extern crate rand;
extern crate num_cpus;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate slab;
extern crate num_bigint;
extern crate num_traits;
extern crate lmdb_zero as lmdb;
#[macro_use]
extern crate lazy_static;
extern crate config;
#[macro_use]
extern crate clap;
extern crate uuid;

extern crate pumpkinscript;
extern crate pumpkindb_engine;

mod connection;
mod server;

use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::thread;
use std::sync::Arc;

use memmap::{Mmap, Protection};
use mio::*;
use mio::tcp::*;
use clap::{App, Arg};

use pumpkindb_engine::{script, storage, timestamp};
use pumpkindb_engine::script::dispatcher;

lazy_static! {
 static ref ENVIRONMENT: lmdb::Environment = {
    let _ = config::set_default("storage.path", "pumpkin.db");
    let storage_path = config::get_str("storage.path").unwrap().into_owned();
    fs::create_dir_all(storage_path.as_str()).expect("can't create directory");
    let map_size = config::get_int("storage.mapsize");
    storage::create_environment(storage_path, map_size)
 };
}

use mio::channel as mio_chan;



pub fn run(port: i64,
           senders: Vec<script::Sender<script::RequestMessage>>,
           relay_sender: mio_chan::Sender<server::RelayedPublishedMessage>,
           relay_receiver: mio_chan::Receiver<server::RelayedPublishedMessage>) {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    info!("Listening on {}", addr);

    let sock = TcpListener::bind(&addr).expect("Failed to bind address");

    let mut poll = Poll::new().expect("Failed to initialize polling");

    let mut server = server::Server::new(sock, relay_sender, relay_receiver, senders);
    server.run(&mut poll).expect("Failed to run server");

}

/// Accepts storage path, filename and length and prepares the file. It is important that the length
/// is the total length of the memory mapped file, otherwise the application _will segfault_ when
/// trying to read those sections later. There is no way to handle that.
/// The mmap file is structured as such now:
/// Byte Range         Used for
/// [0..20]            Last known HTC timestamp
fn prepare_mmap(storage_path: &str, filename: &str, length: u64) -> Mmap {
    let mut scratchpad_pathbuf = PathBuf::from(storage_path);
    scratchpad_pathbuf.push(filename);
    scratchpad_pathbuf.set_extension("dat");
    let scratchpad_path = scratchpad_pathbuf.as_path();
    let scratchpad_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(scratchpad_path)
        .expect("Could not open or create scratchpad");
    let _ = scratchpad_file.set_len(length);
    Mmap::open_path(scratchpad_path, Protection::ReadWrite).expect("Could not open scratchpad")
}

fn main() {
    let args = App::new("PumpkinDB Server")
        .version(crate_version!())
        .about("Event Sourcing Database Engine http://pumpkindb.org")
        .author("PumpkinDB Contributors")
        .setting(clap::AppSettings::ColoredHelp)
        .arg(Arg::with_name("config")
            .help("Configuration file")
            .required(false)
            .long("config")
            .short("c")
            .default_value("pumpkindb.toml")
            .takes_value(true))
        .get_matches();
    let _ = config::merge(config::Environment::new("pumpkindb"));
    let _ = config::merge(config::File::new(args.value_of("config").unwrap(),
                                            config::FileFormat::Toml));
    let _ = config::set_default("server.port", 9981);
    let _ = config::set_default("storage.path", "pumpkin.db");
    let storage_path = config::get_str("storage.path").unwrap().into_owned();
    fs::create_dir_all(storage_path.as_str()).expect("can't create directory");

    // Initialize Mmap
    let scratchpad = prepare_mmap(storage_path.as_str(), "scratchpad", 20);
    let mut hlc_state = scratchpad.into_view_sync();
    hlc_state.restrict(0, 20).expect("Could not prepare HLC state");

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
        let root =
            log4rs::config::Root::builder().appender("console").build(log::LogLevelFilter::Info);
        let _ = log4rs::init_config(log4rs::config::Config::builder()
            .appender(appender)
            .build(root)
            .unwrap());
        warn!("No logging configuration specified, switching to console logging");
    }

    info!("Starting up");

    let mut senders = Vec::new();

    let (relay_sender, relay_receiver) = mio_chan::channel();
    let mut client_messaging = pumpkindb_engine::messaging::Simple::new();
    let publisher_accessor = client_messaging.accessor();
    let subscriber_accessor = client_messaging.accessor();
    let _ = thread::spawn(move || client_messaging.run());
    let storage = Arc::new(storage::Storage::new(&ENVIRONMENT));
    let timestamp = Arc::new(timestamp::Timestamp::new(Some(hlc_state)));

    for i in 0..num_cpus::get() {
        info!("Starting scheduler on core {}.", i);
        let (sender, receiver) = script::Scheduler::<dispatcher::StandardDispatcher>::create_sender();
        let storage_clone = storage.clone();
        let timestamp_clone = timestamp.clone();

        let publisher_accessor1 = publisher_accessor.clone();
        let subscriber_accessor1 = subscriber_accessor.clone();

        thread::spawn(move || {
            let mut scheduler =
                script::Scheduler::new(
                    dispatcher::StandardDispatcher::new(&storage_clone,
                                                        publisher_accessor1, subscriber_accessor1,
                                                        timestamp_clone),
                    receiver);
            scheduler.run()
        });
        senders.push(sender)
    }

    run(config::get_int("server.port").unwrap(),
                senders, relay_sender, relay_receiver);

}
