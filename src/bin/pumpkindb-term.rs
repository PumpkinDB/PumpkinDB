// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate pumpkindb;

extern crate config;
extern crate nom;
extern crate rustyline;
extern crate ansi_term;
extern crate uuid;

use pumpkindb::script;

extern crate byteorder;

use std::io::prelude::*;
use std::net::TcpStream;

use byteorder::{ByteOrder, BigEndian};


use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::fmt::Write;
use std::io::Write as IoWrite;

use ansi_term::Colour::Red;

use std::str;

use uuid::Uuid;

fn main() {
    let _ = config::merge(config::Environment::new("pumpkindb"));
    let _ = config::set_default("prompt", "PumpkinDB> ");

    let mut stream = TcpStream::connect("0.0.0.0:9981").unwrap();

    let mut rl = Editor::<()>::new();

    let mut r = Vec::new();

    loop {
        match rl.readline(format!("{}", config::get_str("prompt").unwrap()).as_str()) {
            Ok(text) => {
                rl.add_history_entry(&text);
                let uuid = Uuid::new_v4();
                match script::parse(format!("[{}] TRY \"{}\" 'topic SET topic SUBSCRIBE STACK topic SEND topic UNSUBSCRIBE",
                                            text, uuid.hyphenated().to_string()).as_str()) {
                    Ok(compiled) => {
                        let msg = compiled;
                        let mut buf = [0u8; 8];

                        BigEndian::write_u64(&mut buf, msg.len() as u64);
                        stream.write_all(buf.as_ref()).unwrap();
                        stream.write_all(msg.as_ref()).unwrap();

                        let mut buf = [0u8; 8];
                        stream.read(&mut buf).unwrap();

                        let msg_len = BigEndian::read_u64(&mut buf);

                        let s_ref = <TcpStream as Read>::by_ref(&mut stream);

                        r.clear();

                        match s_ref.take(msg_len).read_to_end(&mut r) {
                            Ok(0) => {
                            },
                            Ok(_) => {
                                let mut input = r.clone();
                                let mut top_level = true;
                                let mut s = String::new();
                                while input.len()> 0 {
                                    match script::binparser::data(input.clone().as_slice()) {
                                        nom::IResult::Done(rest, data) => {
                                            let (_, size) = script::binparser::data_size(data).unwrap();
                                            let data = &data[script::offset_by_size(size)..];

                                            input = Vec::from(rest);

                                            if rest.len() == 0 && top_level {
                                                top_level = false;
                                                if data.len() > 0 {
                                                    let _ = write!(&mut s, "{}", Red.paint("Error: "));
                                                    input = Vec::from(data);
                                                }
                                            } else {
                                                if data.iter().all(|c| *c >= 0x20 && *c <= 0x7e) {
                                                    let _ = write!(&mut s, "{:?} ", str::from_utf8(data).unwrap());
                                                } else {
                                                    let _ = write!(&mut s, "0x");
                                                    for b in Vec::from(data) {
                                                        let _ = write!(&mut s, "{:02x}", b);
                                                    }
                                                    let _ = write!(&mut s, " ");
                                                }
                                            }
                                        },
                                        e => {
                                            panic!("{:?}", e);
                                        }
                                    }
                                }
                                println!("{}", s);
                            },
                            Err(e) => {
                                panic!("{}", e);
                            }
                        }
                    },
                    Err(err) => {
                        println!("Script error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Aborted");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Exiting");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }


    }
}