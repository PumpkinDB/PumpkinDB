// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


use script;

use mio::*;
use mio::tcp::*;

mod server;
mod connection;
use self::server::*;

use pubsub;

pub fn run(port: i64, sender: script::Sender<script::RequestMessage<'static>>,
                        publisher: pubsub::PublisherAccessor<Vec<u8>>) {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    println!("Listening on {}", addr);

    let sock = TcpListener::bind(&addr).expect("Failed to bind address");

    let mut poll = Poll::new().expect("Failed to initialize polling");

    let mut server = Server::new(sock, sender, publisher);
    server.run(&mut poll).expect("Failed to run server");

}
