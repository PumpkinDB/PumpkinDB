// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This program is used to run doctests defined in doc/script
//
#![feature(slice_patterns)]

extern crate glob;
extern crate regex;
extern crate crossbeam;
extern crate tempdir;

extern crate pumpkinscript;
extern crate pumpkindb_engine;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::sync::mpsc;
use std::sync::Arc;

use regex::Regex;
use glob::glob;
use tempdir::TempDir;

use pumpkindb_engine::script::{SchedulerHandle, ResponseMessage, EnvId, Env, Scheduler, dispatcher};
use pumpkinscript::{textparser, binparser};
use pumpkindb_engine::{messaging, storage, timestamp, nvmem, lmdb};

fn eval(name: &[u8], script: &[u8], timestamp: Arc<timestamp::Timestamp<nvmem::MmapedRegion>>) {
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
    let db = Arc::new(storage::Storage::new(&env));
    crossbeam::scope(|scope| {
        let mut simple = messaging::Simple::new();
        let simple_accessor = simple.accessor();
        let publisher_thread = scope.spawn(move || simple.run());
        let publisher_clone = simple_accessor.clone();
        let subscriber_clone = simple_accessor.clone();
        let timestamp_clone = timestamp.clone();
        let (mut scheduler, sender) = Scheduler::new(
            dispatcher::StandardDispatcher::new(db.clone(), publisher_clone, subscriber_clone,
                                                timestamp_clone));
        let handle = scope.spawn(move || scheduler.run());
        let (callback, receiver) = mpsc::channel::<ResponseMessage>();
        let (sender0, _) = mpsc::channel();
        sender.schedule_env(EnvId::new(), Vec::from(script), callback,
                                                        Box::new(sender0));
        match receiver.recv() {
            Ok(ResponseMessage::EnvTerminated(_, stack, _)) => {
                sender.shutdown();
                simple_accessor.shutdown();
                let mut stack_ = Vec::with_capacity(stack.len());
                for i in 0..(&stack).len() {
                    stack_.push((&stack[i]).as_slice());
                }
                let mut script_env = Env::new_with_stack(stack_).unwrap();
                let val = script_env.pop().unwrap();
                assert_eq!(Vec::from(val),
                           vec![1],
                           "{} was expected to succeeed",
                           &name);
                println!(" * {}", &name);
            }
            Ok(ResponseMessage::EnvFailed(_, err, _, _)) => {
                sender.shutdown();
                simple_accessor.shutdown();
                panic!("Error while executing {:?}: {:?}", &name, err)
            }
            Err(err) => {
                sender.shutdown();
                simple_accessor.shutdown();
                panic!("recv error: {:?}", err);
            }
        }
        let _ = handle.join();
        let _ = publisher_thread.join();
    });
}

fn main() {
    let mut nvmem_mmap = nvmem::MmapedFile::new_anonymous(20).unwrap();
    let nvmem_region = nvmem_mmap.claim(20).unwrap();
    let timestamp = Arc::new(timestamp::Timestamp::new(nvmem_region));
    let re = Regex::new(r"```test\r?\n((.+(\r?\n)*)+)```").unwrap();
    for entry in glob("doc/script/**/*.md").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("{}", path.to_str().unwrap());
                let mut f = File::open(&path).expect("can't open file");
                let mut s = String::new();
                f.read_to_string(&mut s).expect("can't read file");
                for cap in re.captures_iter(&s) {
                    let programs = textparser::programs(cap[1].as_ref()).unwrap().1;
                    if programs.len() == 0 {
                        println!(" WARNING: no tests defined in {}", path.to_str().unwrap());
                    }
                    for program in programs {
                        if program.len() > 0 {
                            match binparser::instruction(program.as_slice()) {
                                pumpkinscript::ParseResult::Done(&[0x81, b':', ref rest..], program) => {
                                    eval(&program[1..], rest, timestamp.clone());
                                }
                                other => panic!("test definition parse error {:?}", other),
                            }
                        }
                    }

                }
            }
            Err(_) => (),
        }
    }
}
