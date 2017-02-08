// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![cfg_attr(test, feature(test))]

#![feature(alloc, heap_api)]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate matches;

#[cfg(test)]
extern crate test;

// Parser
#[macro_use]
extern crate nom;

extern crate snowflake;
extern crate lmdb_zero as lmdb;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate crossbeam;


extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

extern crate config;

#[macro_use]
extern crate lazy_static;

pub mod script;
pub mod server;

use std::thread;

use std::fs;

lazy_static! {
 static ref ENV: lmdb::Environment = {
     let _ = config::set_default("storage.path", "pumpkin.db");

     let path = config::get_str("storage.path").unwrap().into_owned();
     fs::create_dir_all(path.as_str()).expect("can't create directory");
     unsafe {
            lmdb::EnvBuilder::new()
                .expect("can't create env builder")
                .open(path.as_str(), lmdb::open::Flags::empty(), 0o600)
                .expect("can't open env")
    }
 };

 static ref DB: lmdb::Database<'static> = lmdb::Database::open(&ENV,
                              None,
                              &lmdb::DatabaseOptions::new(lmdb::db::CREATE))
                              .expect("can't open database");


}

fn main() {
    let _ = config::merge(config::Environment::new("PUMPKINDB"));
    let _ = config::merge(config::File::new("pumpkindb.toml", config::FileFormat::Toml));

    let _ = config::set_default("binary-server.port", 9980);
    let _ = config::set_default("text-server.port", 9981);


    let mut vm = script::VM::new(&ENV, &DB);
    let sender = vm.sender();

    thread::spawn(move || vm.run());

    server::run_plain_server(config::get_int("text-server.port").unwrap(), sender);

}
