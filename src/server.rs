// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::str;
use tokio_core::io::{Codec, EasyBuf};

pub struct LineCodec;

impl Codec for LineCodec {
    type In = String;
    type Out = String;
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        if let Some(i) = buf.as_slice().iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.drain_to(i);

            // Also remove the '\n'
            buf.drain_to(1);

            // Turn this data into a UTF string and return it in a Frame.
            match str::from_utf8(line.as_slice()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }

    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.push(b'\n');
        Ok(())
    }
}

use tokio_proto::pipeline::ServerProto;

pub struct LineProto;

use tokio_core::io::{Io, Framed};

impl<T: Io + 'static> ServerProto<T> for LineProto {
    type Request = String;

    type Response = String;

    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

use tokio_service::Service;

pub struct PlainServer<'a>(script::Sender<script::RequestMessage<'a>>);

use futures::{future, Future, BoxFuture};
use std::fmt::Write;

use std::sync::mpsc;

// Below is work in progress on making the server non-blocking:
//
//use futures::{Poll, Async}
//
//struct ReceiverFuture<T>(script::Receiver<T>);
//
//impl<T> Future for ReceiverFuture<T> {
//    type Item = T;
//    type Error = mpsc::TryRecvError;
//    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//        match self.0.try_recv() {
//            Err(mpsc::TryRecvError::Empty) => Ok(Async::NotReady),
//            Err(err) => Err(err),
//            Ok(data) => Ok(Async::Ready(data)),
//        }
//    }
//}

impl<'a> Service for PlainServer<'a> {
    type Request = String;
    type Response = String;

    type Error = io::Error;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let (callback, receiver): (mpsc::Sender<script::ResponseMessage<'a>>,
                                   mpsc::Receiver<script::ResponseMessage<'a>>) = mpsc::channel();

        let _ = self.0
            .send(script::RequestMessage::ScheduleEnv(script::EnvId::new(),
                                                      script::parse(req.as_str()).unwrap(),
                                                      callback));
        // FIXME:
        // This is far from perfect as we're blocking here
        match receiver.recv() {
            Ok(script::ResponseMessage::EnvFailed(_, error, _, _)) => {
                future::ok(format!("{:?}", error)).boxed()
            }
            Ok(script::ResponseMessage::EnvTerminated(_, stack, _)) => {
                let mut s = String::new();

                for v in stack {
                    let _ = write!(&mut s, "0x");
                    let (_, size) = script::binparser::data_size(v).unwrap();
                    let offset = script::offset_by_size(size);
                    for i in offset..offset + size - 1 {
                        let _ = write!(&mut s, "{:X}", v[i]).unwrap();
                    }
                    let _ = write!(&mut s, " ");
                }
                future::ok(s).boxed()
            }
            _ => future::ok(String::from("UnknownError")).boxed(),
        }


    }
}

use tokio_proto::TcpServer;
use script;
use std::sync::Mutex;


pub fn run_plain_server(port: i64, sender: script::Sender<script::RequestMessage<'static>>) {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    println!("Listening (text form) on {}", addr);

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    let msender = Mutex::new(sender);

    server.serve(move || {
        let sender = msender.lock().unwrap();
        Ok(PlainServer(sender.clone()))
    });
}
