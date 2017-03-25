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

const MAX_PRE_ALLOC: usize = 10000;

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
    read_continuation: Option<u32>,

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
            None => {
                return Ok(None);
            }
            Some(n) => n,
        };

        if msg_len == 0 {
            return Ok(None);
        }

        let msg_len = msg_len as usize;

        let alloc_len = if msg_len > MAX_PRE_ALLOC {
            MAX_PRE_ALLOC
        } else {
            msg_len
        };

        let mut recv_buf: Vec<u8> = Vec::with_capacity(alloc_len);
        unsafe {
            recv_buf.set_len(alloc_len);
        }

        let mut read = 0;

        let sock_ref = <TcpStream as Read>::by_ref(&mut self.sock);

        while msg_len > read {
            let read_next = if read >= MAX_PRE_ALLOC {
                if msg_len - read <= MAX_PRE_ALLOC {
                    recv_buf.resize(msg_len, 0);
                    msg_len - read
                } else {
                    recv_buf.resize(MAX_PRE_ALLOC + read, 0);
                    MAX_PRE_ALLOC
                }
            } else {
                alloc_len
            };
            match sock_ref.take(read_next as u64).read(&mut recv_buf[read..]) {
                Ok(n) => {
                    if n < read_next as usize {
                        return Err(Error::new(ErrorKind::InvalidData, "Did not read enough bytes"));
                    }
                    read += read_next;
                    self.read_continuation = None;
                }
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        self.read_continuation = Some(msg_len as u32);
                        return Ok(None);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(Some(recv_buf.to_vec()))
    }

    fn read_message_length(&mut self) -> io::Result<Option<u32>> {
        if let Some(n) = self.read_continuation {
            return Ok(Some(n));
        }

        let mut buf = [0u8; 4];

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

        if bytes < 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }

        let msg_len = BigEndian::read_u32(buf.as_ref());

        Ok(Some(msg_len))
    }

    pub fn writable(&mut self) -> io::Result<()> {

        self.send_queue
            .pop()
            .ok_or(Error::new(ErrorKind::Other, "Could not pop send queue"))
            .and_then(|buf| {
                match self.write_message_length(&buf) {
                    Ok(None) => {
                        self.send_queue.push(buf);
                        return Ok(());
                    }
                    Ok(Some(())) => {
                        ()
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }

                match self.sock.write(&*buf) {
                    Ok(_) => {
                        self.write_continuation = false;
                        Ok(())
                    }
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
        let mut send_buf = [0u8; 4];
        BigEndian::write_u32(&mut send_buf, len as u32);

        match self.sock.write(&send_buf) {
            Ok(_) => Ok(Some(())),
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
