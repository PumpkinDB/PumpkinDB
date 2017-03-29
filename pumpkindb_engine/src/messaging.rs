// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use std::sync::mpsc;

///
/// `Publisher` handles sending out a message to all
/// current subscribers
///
pub trait Publisher : Sized {
    fn publish(&self, topic: &[u8], message: &[u8]);
}

///
/// `Subscriber` handles management of subscribers
///
pub trait Subscriber : Sized {
    fn subscribe(&self, topic: &[u8], callback: Box<PublishedMessageCallback + Send>) -> Vec<u8>;
    fn unsubscribe(&self, identifier: &[u8]);
}


///
/// `PublishedMessageCallback` trait defines a subscription
/// delivery callback
///
pub trait PublishedMessageCallback {
    fn call(&self, topic: &[u8], message: &[u8]);
    fn cloned(&self) -> Box<PublishedMessageCallback + Send>;
}

impl PublishedMessageCallback for mpsc::Sender<(Vec<u8>, Vec<u8>)> {
    fn call(&self, topic: &[u8], message: &[u8]) {
        let _ = self.send((Vec::from(topic), Vec::from(message)));
    }
    fn cloned(&self) -> Box<PublishedMessageCallback + Send> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use super::PublishedMessageCallback;

    #[test]
    pub fn sender_published_message_callback_call() {
        let (sender, receiver) = mpsc::channel();
        sender.call("Hello".as_bytes(), "world".as_bytes());
        assert_eq!((Vec::from("Hello"), Vec::from("world")), receiver.recv().unwrap());
    }

    #[test]
    pub fn sender_published_message_callback_cloned() {
        let (sender, receiver) = mpsc::channel();
        let sender1 = sender.cloned();
        sender1.call("Hello".as_bytes(), "world".as_bytes());
        assert_eq!((Vec::from("Hello"), Vec::from("world")), receiver.recv().unwrap());
    }
}

mod simple {
    use std::sync::mpsc;
    use std::collections::BTreeMap;

    use uuid::Uuid;

    use super::*;

    pub struct Message { topic: Vec<u8>, body: Vec<u8> }

    pub enum SimpleControlMessage {
        Publish(Message),
        Subscribe {
            topic: Vec<u8>,
            callback: Box<PublishedMessageCallback + Send>,
            identifier: Vec<u8>
        },
        Unsubscribe { identifier: Vec<u8> },
        Shutdown
    }

    pub struct Simple {
        sender: mpsc::Sender<SimpleControlMessage>,
        receiver: mpsc::Receiver<SimpleControlMessage>,
        subscriptions: BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
        identifier_subscription: BTreeMap<Vec<u8>, Box<PublishedMessageCallback + Send>>,
    }

    pub struct SimpleAccessor {
        sender: mpsc::Sender<SimpleControlMessage>,
    }

    impl SimpleAccessor {

        fn send_publish(&self, message: Message) {
            let _ = self.sender.send(SimpleControlMessage::Publish(message));
        }

        fn send_subscribe(&self, topic: Vec<u8>,
                          callback: Box<PublishedMessageCallback + Send>,
                          identifier: Vec<u8>) {
            let _ = self.sender.send(SimpleControlMessage::Subscribe{
                topic: topic,
                callback: callback,
                identifier: identifier,
            });
        }

        fn send_unsubscribe(&self, identifier: Vec<u8>) {
            let _ = self.sender.send(SimpleControlMessage::Unsubscribe {identifier: identifier});
        }

        pub fn shutdown(&self) {
            let _ = self.sender.send(SimpleControlMessage::Shutdown);
        }

    }

    impl Clone for SimpleAccessor {
        fn clone(&self) -> Self {
            SimpleAccessor{ sender: self.sender.clone() }
        }
    }

    impl Simple {
        pub fn new() -> Self {
            let (sender, receiver) = mpsc::channel();
            Simple {
                sender: sender,
                receiver: receiver,
                subscriptions: BTreeMap::new(),
                identifier_subscription: BTreeMap::new(),
            }
        }

        pub fn accessor(&self) -> SimpleAccessor {
            SimpleAccessor{ sender: self.sender.clone() }
        }

        pub fn run(&mut self) {
            loop {
                match self.receiver.recv() {
                    Ok(SimpleControlMessage::Publish(message)) =>
                        match self.subscriptions.get_mut(&message.topic) {
                            Some(subscribers) => for identifier in subscribers {
                                match self.identifier_subscription.get(identifier) {
                                    None => (),
                                    Some(subscriber) =>
                                        subscriber.call(&message.topic, &message.body)
                                }
                            },
                            None => continue
                        },
                    Ok(SimpleControlMessage::Subscribe {topic, callback, identifier}) => {
                        if !self.subscriptions.contains_key(&topic) {
                            self.subscriptions.insert(topic.clone(), vec![]);
                        }
                        let subscribers = self.subscriptions.get_mut(&topic).unwrap();
                        subscribers.push(identifier.clone());
                        self.identifier_subscription.insert(identifier, callback);
                    },
                    Ok(SimpleControlMessage::Unsubscribe {identifier}) => {
                        self.identifier_subscription.remove(&identifier);
                    },
                    Ok(SimpleControlMessage::Shutdown) => break,
                    Err(_) => break
                }
            }
        }
    }

    impl Publisher for SimpleAccessor {
        fn publish(&self, topic: &[u8], message: &[u8]) {
            self.send_publish(Message {
                topic: Vec::from(topic),
                body: Vec::from(message)
            })
        }
    }

    impl Subscriber for SimpleAccessor {
        fn subscribe(&self, topic: &[u8], callback: Box<PublishedMessageCallback + Send>) -> Vec<u8> {
            let uuid = Uuid::new_v4();
            let id = uuid.as_bytes()[..].to_vec();
            self.send_subscribe(Vec::from(topic), callback, id.clone());
            id
        }
        fn unsubscribe(&self, identifier: &[u8]) {
            self.send_unsubscribe(identifier.to_vec())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::thread;
        use std::sync::mpsc;
        use std::time::Duration;

        use super::*;

        #[test]
        pub fn subscribe() {
            let mut simple = Simple::new();
            let accessor = simple.accessor();
            thread::spawn(move || simple.run());

            let (sender, receiver) = mpsc::channel();
            let _ = accessor.subscribe("test".as_bytes(), Box::new(sender));

            accessor.publish("test".as_bytes(), "hello".as_bytes());

            assert_eq!(receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
                       (Vec::from("test"), Vec::from("hello")));

            accessor.shutdown();
        }

        #[test]
        pub fn unsubscribe() {
            let mut simple = Simple::new();
            let accessor = simple.accessor();
            thread::spawn(move || simple.run());

            let (sender, receiver) = mpsc::channel();
            // we are cloning sender here so that the error check
            // below doesn't result in `Disconnected` but `Timeout`
            let subscription = accessor.subscribe("test".as_bytes(), Box::new(sender.clone()));

            accessor.unsubscribe(&subscription);

            accessor.publish("test".as_bytes(), "hello".as_bytes());

            assert_eq!(receiver.recv_timeout(Duration::from_secs(1)).unwrap_err(),
                       mpsc::RecvTimeoutError::Timeout);

            accessor.shutdown();
        }
    }

}

pub use self::simple::{Simple, SimpleAccessor};