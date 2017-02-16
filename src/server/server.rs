// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::rc::Rc;

use mio::*;
use mio::tcp::*;
use slab;

use super::connection::Connection;

type Slab<T> = slab::Slab<T, Token>;

use script;
use script::{EnvId, Sender, RequestMessage, ResponseMessage, parse};
use nom;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use std::sync::mpsc;

use pubsub;

use std::thread;
use std::sync::Mutex;

use mio::channel as mio_chan;

use std::collections::BTreeMap;

pub struct Server {
    sender: Sender<RequestMessage<'static>>,
    response_sender: Sender<ResponseMessage<'static>>,
    evented_sender: mio_chan::Sender<(Token, Vec<u8>, Vec<u8>)>,
    receiver: mio_chan::Receiver<(Token, Vec<u8>, Vec<u8>)>,
    publisher: pubsub::PublisherAccessor<Vec<u8>>,
    sock: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    events: Events,
}


impl Server {
    pub fn new(sock: TcpListener, sender: Sender<RequestMessage<'static>>, publisher: pubsub::PublisherAccessor<Vec<u8>>) -> Server {
        let (response_sender, _) = mpsc::channel();
        let (evented_sender, receiver) = mio_chan::channel();

        Server {
            sock: sock,
            sender: sender,
            response_sender: response_sender,
            evented_sender: evented_sender,
            receiver: receiver,
            publisher: publisher,
            token: Token(10_000_000),
            conns: Slab::with_capacity(128),
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
            let mut map = BTreeMap::new();

            loop {
                match receiver.recv() {
                    Ok((original_topic, message, callback)) => {
                        if original_topic == "subscriptions".as_bytes() || original_topic == "unsubscriptions".as_bytes()  {
                            let mut input = Vec::from(message);
                            let topic = match script::binparser::data(input.clone().as_slice()) {
                                nom::IResult::Done(rest, data) => {
                                    let (_, size) = script::binparser::data_size(data).unwrap();
                                    input = Vec::from(rest);
                                    Vec::from(&data[script::offset_by_size(size)..])
                                }
                                _ => continue
                            };
                            let token = Token(match script::binparser::data(input.clone().as_slice()) {
                                nom::IResult::Done(_, data) => {
                                    let (_, size) = script::binparser::data_size(data).unwrap();
                                    BigUint::from_bytes_be(&data[script::offset_by_size(size)..]).to_u64().unwrap()
                                }
                                _ => continue
                            } as usize);
                            if original_topic == "subscriptions".as_bytes() {
                                if !map.contains_key(&topic) {
                                    map.insert(topic.clone(), Vec::new());
                                }
                                let mut tokens = map.remove(&topic).unwrap();
                                tokens.push(token);
                                map.insert(topic.clone(), tokens);
                                publisher.subscribe(topic, sender.clone());
                            } else {
                                if map.contains_key(&topic) {
                                    let tokens = map.remove(&topic).unwrap();
                                    let new_tokens = tokens.into_iter().filter(|t| t.0 != token.0).collect();
                                    map.insert(topic.clone(), new_tokens);
                                }
                            }
                            let _ = callback.send(());
                        } else {
                            let _ = callback.send(());
                            if map.contains_key(&original_topic) {
                                for token in map.get(&original_topic).unwrap() {
                                    let _ = evented_sender.send((*token, original_topic.clone(), message.clone()));
                                }
                            }
                        }
                    },
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
            }
        });

        let _ = poll.register(
            &self.receiver,
            Token(1000000),
            Ready::all(),
            PollOpt::edge()
        ).or_else(|e| {
            Err(e)
        });

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
        poll.register(
            &self.sock,
            self.token,
            Ready::readable(),
            PollOpt::edge()
        ).or_else(|e| {
            Err(e)
        })
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
        }
    }

    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready) {
        if token == Token(1000000) {
            let (token, _, msg) = self.receiver.try_recv().unwrap();
            let _ = self.find_connection_by_token(token).send_message(Rc::new(msg));
            self.find_connection_by_token(token).mark_idle();
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
                .unwrap_or_else(|_| {
                    conn.mark_reset();
                });
        }

        if event.is_readable() {
            if self.token == token {
                self.accept(poll);
            } else {

                if self.find_connection_by_token(token).is_reset() {
                    return;
                }

                self.readable(token)
                    .unwrap_or_else(|_| {
                        self.find_connection_by_token(token).mark_reset();
                    });
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
                Err(_) => return
            };

            let token = match self.conns.vacant_entry() {
                Some(entry) => {
                    let c = Connection::new(sock, entry.index());
                    entry.insert(c).index()
                }
                None => return
            };

            match self.find_connection_by_token(token).register(poll) {
                Ok(_) => {},
                Err(_) => {
                    self.conns.remove(token);
                }
            }
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        while let Some(mut message) = self.find_connection_by_token(token).readable()? {

            let id = EnvId::new();
            let mut vec = Vec::new();
            let mut subscribe = parse(format!("[{} 2 WRAP \"subscriptions\" SEND] 'SUBSCRIBE DEF", token.0).as_str()).unwrap();
            let mut unsubscribe = parse(format!("[{} 2 WRAP \"unsubscriptions\" SEND] 'UNSUBSCRIBE DEF", token.0).as_str()).unwrap();
            vec.append(&mut subscribe);
            vec.append(&mut unsubscribe);
            vec.append(&mut message);
            let _ = self.sender.send(RequestMessage::ScheduleEnv(id, Vec::from(vec), self.response_sender.clone()));

        }

        Ok(())
    }

    fn find_connection_by_token(&mut self, token: Token) -> &mut Connection {
        &mut self.conns[token]
    }
}