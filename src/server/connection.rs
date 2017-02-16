// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use byteorder::{ByteOrder, BigEndian};

use mio::*;
use mio::tcp::*;

pub struct Connection {
    // handle to the accepted socket
    sock: TcpStream,

    // token used to register with the poller
    pub token: Token,

    // set of events we are interested in
    interest: Ready,

    // messages waiting to be sent out
    send_queue: Vec<Rc<Vec<u8>>>,

    // track whether a connection needs to be (re)registered
    is_idle: bool,

    // track whether a connection is reset
    is_reset: bool,

    // track whether a read received `WouldBlock` and store the number of
    // byte we are supposed to read
    read_continuation: Option<u64>,

    // track whether a write received `WouldBlock`
    write_continuation: bool,

}

impl Connection {
    pub fn new(sock: TcpStream, token: Token) -> Connection {
        Connection {
            sock: sock,
            token: token,
            interest: Ready::hup(),
            send_queue: Vec::new(),
            is_idle: true,
            is_reset: false,
            read_continuation: None,
            write_continuation: false,
        }
    }

    pub fn readable(&mut self) -> io::Result<Option<Vec<u8>>> {

        let msg_len = match try!(self.read_message_length()) {
            None => { return Ok(None); },
            Some(n) => n,
        };

        if msg_len == 0 {
            return Ok(None);
        }

        let msg_len = msg_len as usize;

        let mut recv_buf : Vec<u8> = Vec::with_capacity(msg_len);
        unsafe { recv_buf.set_len(msg_len); }

        let sock_ref = <TcpStream as Read>::by_ref(&mut self.sock);

        match sock_ref.take(msg_len as u64).read(&mut recv_buf) {
            Ok(n) => {
                if n < msg_len as usize {
                    return Err(Error::new(ErrorKind::InvalidData, "Did not read enough bytes"));
                }

                self.read_continuation = None;

                Ok(Some(recv_buf.to_vec()))
            }
            Err(e) => {

                if e.kind() == ErrorKind::WouldBlock {
                    self.read_continuation = Some(msg_len as u64);
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    fn read_message_length(&mut self) -> io::Result<Option<u64>> {
        if let Some(n) = self.read_continuation {
            return Ok(Some(n));
        }

        let mut buf = [0u8; 8];

        let bytes = match self.sock.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };

        if bytes < 8 {
            //println!("Found message length of {} bytes", bytes);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }

        let msg_len = BigEndian::read_u64(buf.as_ref());
        Ok(Some(msg_len))
    }

    pub fn writable(&mut self) -> io::Result<()> {

        self.send_queue.pop()
            .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {
                match self.write_message_length(&buf) {
                    Ok(None) => {
                        self.send_queue.push(buf);
                        return Ok(());
                    },
                    Ok(Some(())) => {
                        ()
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }

                match self.sock.write(&*buf) {
                    Ok(_) => {
                        self.write_continuation = false;
                        Ok(())
                    },
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            self.send_queue.push(buf);
                            self.write_continuation = true;
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            })?;

        if self.send_queue.is_empty() {
            self.interest.remove(Ready::writable());
        }

        Ok(())
    }

    fn write_message_length(&mut self, buf: &Rc<Vec<u8>>) -> io::Result<Option<()>> {
        if self.write_continuation {
            return Ok(Some(()));
        }

        let len = buf.len();
        let mut send_buf = [0u8; 8];
        BigEndian::write_u64(&mut send_buf, len as u64);

        match self.sock.write(&send_buf) {
            Ok(_) => {
                Ok(Some(()))
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn send_message(&mut self, message: Rc<Vec<u8>>) -> io::Result<()> {
        self.send_queue.push(message);

        if !self.interest.is_writable() {
            self.interest.insert(Ready::writable());
        }

        Ok(())
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        self.interest.insert(Ready::readable());

        poll.register(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).and_then(|(),| {
            self.is_idle = false;
            Ok(())
        }).or_else(|e| {
            Err(e)
        })
    }

    pub fn reregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.reregister(
            &self.sock,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
        ).and_then(|(),| {
            self.is_idle = false;
            Ok(())
        }).or_else(|e| {
            Err(e)
        })
    }

    pub fn mark_reset(&mut self) {
        self.is_reset = true;
    }

    #[inline]
    pub fn is_reset(&self) -> bool {
        self.is_reset
    }

    pub fn mark_idle(&mut self) {
        self.is_idle = true;
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.is_idle
    }
}