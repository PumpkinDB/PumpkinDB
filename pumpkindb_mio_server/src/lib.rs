// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]

extern crate mio;
extern crate memmap;
extern crate byteorder;
extern crate rand;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate slab;
extern crate num_bigint;
extern crate num_traits;
extern crate uuid;

extern crate pumpkindb_engine;

mod connection;
mod server;

use mio::Poll;
use mio::tcp::TcpListener;

use mio::channel as mio_chan;

use pumpkindb_engine::{script};

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

