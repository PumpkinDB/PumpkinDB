// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! This program is used to run doctests defined in doc/script
//!
#![feature(slice_patterns)]
extern crate glob;
extern crate pumpkindb;
extern crate regex;
extern crate crossbeam;
extern crate tempdir;
extern crate lmdb_zero as lmdb;
extern crate nom;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::sync::mpsc;

use regex::Regex;
use glob::glob;

use pumpkindb::script::{RequestMessage, ResponseMessage, EnvId, Env, Scheduler};
use pumpkindb::script::{textparser, binparser};
use pumpkindb::pubsub;
use pumpkindb::storage;
use pumpkindb::timestamp;
use std::sync::Arc;
use tempdir::TempDir;

fn eval(name: &[u8], script: &[u8]) {
    let dir = TempDir::new("pumpkindb").unwrap();
    let path = dir.path().to_str().unwrap();
    fs::create_dir_all(path).expect("can't create directory");
    let env = unsafe {
        lmdb::EnvBuilder::new()
            .expect("can't create env builder")
            .open(path, lmdb::open::NOTLS, 0o600)
            .expect("can't open env")
    };
    let name = String::from(std::str::from_utf8(name).unwrap());
    let db = storage::Storage::new(&env);
    crossbeam::scope(|scope| {
        let timestamp = Arc::new(timestamp::Timestamp::new());
        let mut publisher = pubsub::Publisher::new();
        let publisher_accessor = publisher.accessor();
        let publisher_thread = scope.spawn(move || publisher.run());
        let publisher_clone = publisher_accessor.clone();
        let timestamp_clone = timestamp.clone();
        let (sender_sender, receiver) = mpsc::sync_channel(0);
        let handle = scope.spawn(move || {
            let mut scheduler = Scheduler::new(
                &db,
                publisher_clone,
                timestamp_clone,
                sender_sender,
            );
            scheduler.run()
        });
        let sender = receiver.recv().unwrap();
        let (callback, receiver) = mpsc::channel::<ResponseMessage>();
        let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), Vec::from(script), callback));
        match receiver.recv() {
            Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                let _ = sender.send(RequestMessage::Shutdown);
                publisher_accessor.shutdown();
                let mut stack_ = Vec::with_capacity(stack.len());
                for i in 0..(&stack).len() {
                    stack_.push((&stack[i]).as_slice());
                }
                let mut script_env = Env::new_with_stack(stack_, stack_size).unwrap();
                let val = script_env.pop().unwrap();
                assert_eq!(Vec::from(val), vec![1], "{} was expected to succeeed", &name);
                println!(" * {}", &name);
            }
            Ok(ResponseMessage::EnvFailed(_, err, _, _)) => {
                let _ = sender.send(RequestMessage::Shutdown);
                publisher_accessor.shutdown();
                panic!("Error while executing {:?}: {:?}", &name, err)
            }
            Err(err) => {
               let _ = sender.send(RequestMessage::Shutdown);
               publisher_accessor.shutdown();
               panic!("recv error: {:?}", err);
            }
        }
        let _ = handle.join();
        let _ = publisher_thread.join();
    });
}

fn main() {
    let re = Regex::new(r"```test\r?\n((.+\r?\n?)+)```").unwrap();
    for entry in glob("doc/script/**/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("{}:", path.to_str().unwrap());
                let mut f = File::open(path).expect("can't open file");
                let mut s = String::new();
                f.read_to_string(&mut s).expect("can't read file");
                for cap in re.captures_iter(s.as_ref()) {
                    let programs = textparser::programs(cap[1].as_ref()).unwrap().1;
                    for program in programs {
                        if program.len() > 0 {
                            match binparser::word(program.as_slice()) {
                                nom::IResult::Done(&[0x81, b':', ref rest..], program) => {
                                    eval(&program[1..], rest);
                                },
                                other => panic!("test definition parse error {:?}", other)
                            }
                        }
                    }

                }
            },
            Err(_) => (),
        }
    }
}
