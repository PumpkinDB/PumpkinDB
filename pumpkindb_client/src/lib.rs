// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(fn_traits)]

extern crate byteorder;
extern crate pumpkinscript;

mod packet;
pub use packet::{PacketReader, PacketWriter};

use std::io;
use std::io::{Write, Read};
pub use pumpkinscript::{Encodable, Receivable};

pub trait Send {
    fn send<E : Encodable>(&mut self, encodable: E) -> io::Result<()>;
}

impl<T : Write> Send for PacketWriter<T> {
    fn send<E: Encodable>(&mut self, encodable: E) -> io::Result<()> {
        self.write(&encodable.encode()).map(|_| ())
    }
}

pub trait MessageHandler {
    fn handle_message(&mut self, data: &[u8]);
}

impl<T : FnMut(&[u8])> MessageHandler for T {
    fn handle_message(&mut self, data: &[u8]) {
        self.call_mut((data,))
    }
}

// This trait provides data receiving primitives
pub trait Receive {
    // Receive and handle one message. Returns after one message has
    // been handled
    fn receive<H : MessageHandler>(&mut self, handler: H) -> io::Result<()>;
}

impl<T : Read> Receive for T {
    fn receive<H: MessageHandler>(&mut self, mut handler: H) -> io::Result<()> {
        let mut reader = PacketReader::new(self);
        match reader.read() {
            Ok(data) => {
                handler.handle_message(&data);
                Ok(())
            },
            Err(err) => Err(err),
        }
    }
}
