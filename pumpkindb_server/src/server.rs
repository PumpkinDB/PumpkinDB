// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::sync::Mutex;
use std::collections::BTreeMap;

use slab;
use mio::channel as mio_chan;
use mio::*;
use mio::tcp::*;
use rand::{thread_rng, Rng};

use super::connection::Connection;

type Slab<T> = slab::Slab<T, Token>;

use pumpkinscript::{self, binparser};
use pumpkindb_engine::{script, pubsub};
use pumpkindb_engine::script::{EnvId, Sender, RequestMessage, ResponseMessage};

use uuid::Uuid;

pub struct Server {
    senders: Vec<Sender<RequestMessage>>,
    response_sender: Sender<ResponseMessage>,
    evented_sender: mio_chan::Sender<(Vec<u8>, Vec<u8>, Vec<u8>)>,
    receiver: mio_chan::Receiver<(Vec<u8>, Vec<u8>, Vec<u8>)>,
    publisher: pubsub::PublisherAccessor<Vec<u8>>,
    sock: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    session_token: BTreeMap<Vec<u8>, Token>,
    token_session: BTreeMap<Token, Vec<u8>>,
    events: Events,
}


impl Server {
    pub fn new(sock: TcpListener,
               senders: Vec<Sender<RequestMessage>>,
               publisher: pubsub::PublisherAccessor<Vec<u8>>)
               -> Server {
        let (response_sender, _) = mpsc::channel();
        let (evented_sender, receiver) = mio_chan::channel();

        Server {
            sock: sock,
            senders: senders,
            response_sender: response_sender,
            evented_sender: evented_sender,
            receiver: receiver,
            publisher: publisher,
            token: Token(10_000_000),
            conns: Slab::with_capacity(128),
            session_token: BTreeMap::new(),
            token_session: BTreeMap::new(),
            events: Events::with_capacity(1024),
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {

        self.register(poll)?;

        let publisher = Mutex::new(self.publisher.clone());

        let evented_sender = self.evented_sender.clone();

        thread::spawn(move || {
            let publisher = publisher.lock().unwrap();
            let (sender, receiver) = mpsc::channel();
            publisher.subscribe(Vec::from("subscriptions"), sender.clone());
            publisher.subscribe(Vec::from("unsubscriptions"), sender.clone());
            let mut map = BTreeMap::new();

            loop {
                match receiver.recv() {
                    Ok((original_topic, message, callback)) => {
                        if original_topic == "subscriptions".as_bytes() ||
                           original_topic == "unsubscriptions".as_bytes() {
                            let mut input = Vec::from(message);
                            let topic = match binparser::data(input.clone().as_slice()) {
                                pumpkinscript::ParseResult::Done(rest, data) => {
                                    let (_, size) = binparser::data_size(data).unwrap();
                                    input = Vec::from(rest);
                                    Vec::from(&data[pumpkinscript::offset_by_size(size)..])
                                }
                                _ => continue,
                            };
                            let session = match binparser::data(&input) {
                                pumpkinscript::ParseResult::Done(_, data) => {
                                    let (_, size) = binparser::data_size(data).unwrap();
                                    Vec::from(&data[script::offset_by_size(size)..])
                                },
                                _ => continue
                            };
                            if original_topic == "subscriptions".as_bytes() {
                                let subscribed = map.contains_key(&topic);
                                if !subscribed {
                                    map.insert(topic.clone(), Vec::new());
                                }
                                let mut sessions = map.remove(&topic).unwrap();
                                sessions.push(session);
                                map.insert(topic.clone(), sessions);
                                if !subscribed {
                                    publisher.subscribe(topic, sender.clone());
                                }
                            } else {
                                if map.contains_key(&topic) {
                                    let sessions = map.remove(&topic).unwrap();
                                    let new_sessions: Vec<Vec<u8>> =
                                        sessions.into_iter().filter(|s| *s != session).collect();
                                    map.insert(topic.clone(), new_sessions);
                                }
                            }
                            let _ = callback.send(());
                        } else {
                            let _ = callback.send(());
                            if map.contains_key(&original_topic) {
                                for session in map.get(&original_topic).unwrap() {
                                    let _ =
                                        evented_sender.send((session.clone(),
                                                             original_topic.clone(),
                                                             message.clone()));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("{:?}", err);
                    }
                }
            }
        });

        let _ = poll.register(&self.receiver,
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
            let (session, _, msg) = self.receiver.try_recv().unwrap();
            let _ = poll.reregister(&self.receiver,
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
        use pumpkinscript::compose::Item::*;
        use pumpkinscript::compose::Program;
        while let Some(mut message) = self.find_connection_by_token(token).readable()? {
            let id = EnvId::new();
            let session = self.token_session.get(&token).unwrap();
            let subscribe_def: Vec<u8> = Program(vec![
                Data(&session),
                Data(&[2]), Instruction("WRAP"),
                Data("subscriptions".as_bytes()), Instruction("SEND")
            ]).into();
            let mut subscribe: Vec<u8> = Program(vec![
                Data(&subscribe_def),
                InstructionRef("SUBSCRIBE"), Instruction("DEF")]).into();

            let unsubscribe_def: Vec<u8> = Program(vec![
                Data(&session),
                Data(&[2]), Instruction("WRAP"),
                Data("unsubscriptions".as_bytes()), Instruction("SEND")
            ]).into();
            let mut unsubscribe: Vec<u8> = Program(vec![
                Data(&unsubscribe_def),
                InstructionRef("UNSUBSCRIBE"), Instruction("DEF")]).into();

            let mut vec = Vec::new();
            vec.append(&mut subscribe);
            vec.append(&mut unsubscribe);
            vec.append(&mut message);

            let mut rng = thread_rng();
            let index: usize = rng.gen_range(0, self.senders.len() - 1);
            let sender = self.senders.get(index);
            let _ = sender.unwrap()
                .send(RequestMessage::ScheduleEnv(id,
                                                  Vec::from(vec),
                                                  self.response_sender.clone()));

        }

        Ok(())
    }

    fn find_connection_by_token(&mut self, token: Token) -> &mut Connection {
        &mut self.conns[token]
    }

}
