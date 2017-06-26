// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::rc::Rc;
use std::sync::mpsc;
use std::collections::BTreeMap;

use slab;
use mio::channel as mio_chan;
use mio::*;
use mio::tcp::*;

use super::connection::Connection;

type Slab<T> = slab::Slab<T, Token>;

use pumpkindb_engine::messaging;
use pumpkindb_engine::script::{EnvId, Sender, RequestMessage, ResponseMessage, SchedulerHandle};

use uuid::Uuid;

pub type RelayedPublishedMessage = (Vec<u8>, Vec<u8>, Vec<u8>);

struct RelayedPublishedMessageSender {
    identifier: Vec<u8>,
    sender: mio_chan::Sender<RelayedPublishedMessage>
}

impl messaging::PublishedMessageCallback for RelayedPublishedMessageSender  {
    fn call(&self, topic: &[u8], message: &[u8]) {
        let _ = self.sender.send((self.identifier.clone(), topic.to_vec(), message.to_vec()));
    }

    fn cloned(&self) -> Box<messaging::PublishedMessageCallback + Send> {
        Box::new(RelayedPublishedMessageSender{
            identifier: self.identifier.clone(),
            sender: self.sender.clone(),
        })
    }
}

pub struct Server {
    senders: Vec<Sender<RequestMessage>>,
    response_sender: Sender<ResponseMessage>,
    relay_sender: mio_chan::Sender<RelayedPublishedMessage>,
    relay_receiver: mio_chan::Receiver<RelayedPublishedMessage>,
    sock: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    session_token: BTreeMap<Vec<u8>, Token>,
    token_session: BTreeMap<Token, Vec<u8>>,
    events: Events,
}

impl Server {
    pub fn new(sock: TcpListener,
               relay_sender: mio_chan::Sender<RelayedPublishedMessage>,
               relay_receiver: mio_chan::Receiver<RelayedPublishedMessage>,
               senders: Vec<Sender<RequestMessage>>)
               -> Server {
        let (response_sender, _) = mpsc::channel();

        Server {
            sock: sock,
            senders: senders,
            response_sender: response_sender,
            relay_sender: relay_sender,
            relay_receiver: relay_receiver,
            token: Token(10_000_000),
            conns: Slab::with_capacity(128),
            session_token: BTreeMap::new(),
            token_session: BTreeMap::new(),
            events: Events::with_capacity(1024),
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {

        self.register(poll)?;

        let _ = poll.register(&self.relay_receiver,
                              Token(1000000),
                              Ready::all(),
                              PollOpt::edge())
            .or_else(|e| Err(e));

        loop {
            let cnt = poll.poll(&mut self.events, None)?;

            let mut i = 0;

            while i < cnt {
                let event = self.events.get(i).expect("Failed to get event");

                self.ready(poll, event.token(), event.kind());

                i += 1;
            }

            self.tick(poll);
        }
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.register(&self.sock, self.token, Ready::readable(), PollOpt::edge())
            .or_else(|e| Err(e))
    }

    fn tick(&mut self, poll: &mut Poll) {
        let mut reset_tokens = Vec::new();

        for c in self.conns.iter_mut() {
            if c.is_reset() {
                reset_tokens.push(c.token);
            } else if c.is_idle() {
                c.reregister(poll)
                    .unwrap_or_else(|_| {
                        c.mark_reset();
                        reset_tokens.push(c.token);
                    });
            }
        }

        for token in reset_tokens {
            self.conns.remove(token);
            if let Some(session) = self.token_session.remove(&token) {
                let _ = self.session_token.remove(&session);
            }
        }
    }

    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready) {
        if token == Token(1000000) {
            let (session, _, msg) = self.relay_receiver.try_recv().unwrap();
            let _ = poll.reregister(&self.relay_receiver,
                                    Token(1000000),
                                    Ready::all(),
                                    PollOpt::edge());
            let target = match self.session_token.get(&session) {
                Some(target) => target.clone(),
                None => return
            };
            let conn = self.find_connection_by_token(target);
            let _ = conn.send_message(Rc::new(msg));
            conn.mark_idle();
            return;
        }

        if event.is_error() {
            self.find_connection_by_token(token).mark_reset();
            return;
        }

        if event.is_hup() {
            self.find_connection_by_token(token).mark_reset();
            return;
        }

        if event.is_writable() {
            assert!(self.token != token, "Received writable event for Server");

            let conn = self.find_connection_by_token(token);

            if conn.is_reset() {
                return;
            }

            let _ = conn.writable()
                .unwrap_or_else(|_| { conn.mark_reset(); });
        }

        if event.is_readable() {
            if self.token == token {
                self.accept(poll);
            } else {

                if self.find_connection_by_token(token).is_reset() {
                    return;
                }

                self.readable(token)
                    .unwrap_or_else(|_| { self.find_connection_by_token(token).mark_reset(); });
            }
        }

        if self.token != token {
            self.find_connection_by_token(token).mark_idle();
        }
    }

    fn accept(&mut self, poll: &mut Poll) {
        loop {
            let sock = match self.sock.accept() {
                Ok((sock, _)) => sock,
                Err(_) => return,
            };

            let token = match self.conns.vacant_entry() {
                Some(entry) => {
                    let c = Connection::new(sock, entry.index());
                    entry.insert(c).index()
                }
                None => return,
            };

            match self.find_connection_by_token(token).register(poll) {
                Ok(_) => {
                    let session = Vec::from(&Uuid::new_v4().as_bytes()[..]);
                    self.session_token.insert(session.clone(), token);
                    self.token_session.insert(token, session);
                }
                Err(_) => {
                    self.conns.remove(token);
                }
            }
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        while let Some(message) = self.find_connection_by_token(token).readable()? {
            let id = EnvId::new();
            let session = self.token_session.get(&token).unwrap();
            let _ = self.senders.schedule_env(id,
                                                 message,
                                                 self.response_sender.clone(),
                                                 Box::new(RelayedPublishedMessageSender {
                                                     identifier: session.to_vec(),
                                                     sender: self.relay_sender.clone(),
                                                 }));
        }

        Ok(())
    }

    fn find_connection_by_token(&mut self, token: Token) -> &mut Connection {
        &mut self.conns[token]
    }

}
