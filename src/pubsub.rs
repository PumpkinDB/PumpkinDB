// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Pubsub
//!
//! So why does a database need a publish-subcriber mechanism? A few reasons. Originally,
//! it was started as a way to communicate with connected clients so that they can receive
//! streams of data instead of large stacks.
//!
//! But it is also very useful for the mechanism of subscriptions. For example, what if we
//! sent every event journalled into a topic of some kind and processed it there? This would
//! open some really interesting opportunities.
//!

use std::sync::mpsc;
use std::collections::BTreeMap;

pub type Topic = Vec<u8>;
pub type SubscriberSender<T> = mpsc::Sender<(Topic, T, mpsc::Sender<()>)>;
pub type SubscriberReceiver<T> = mpsc::Receiver<(Topic, T)>;


/// Main entry point for fanning data out
pub struct Publisher<T: Sized + Clone> {
    receiver: mpsc::Receiver<PublisherMessage<T>>,
    sender: mpsc::Sender<PublisherMessage<T>>,
    subscribers: BTreeMap<Topic, Vec<SubscriberSender<T>>>,
}

/// Message types supported by the publisher
enum PublisherMessage<T: Sized + Clone> {
    Subscribe(Topic, SubscriberSender<T>),
    Send(Topic, T, mpsc::Sender<()>),
    Shutdown,
}

impl<T: Sized + Clone> Publisher<T> {
    /// Creates a new Publisher
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Publisher {
            sender: sender,
            receiver: receiver,
            subscribers: BTreeMap::new(),
        }
    }

    /// Creates a cloneable accessor to the publisher
    pub fn accessor(&self) -> PublisherAccessor<T> {
        PublisherAccessor::new(self.sender.clone())
    }

    /// Publisher thread loop
    pub fn run(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(PublisherMessage::Shutdown) => break,
                Ok(PublisherMessage::Subscribe(topic, chan)) => {
                    if !self.subscribers.contains_key(&topic) {
                        self.subscribers.insert(topic.clone(), Vec::new());
                    }
                    self.subscribers.get_mut(&topic).unwrap().push(chan);
                }
                Ok(PublisherMessage::Send(topic, data, callback)) => {
                    if self.subscribers.contains_key(&topic) {
                        let subscribers = self.subscribers.remove(&topic).unwrap();
                        let new_subscribers = subscribers.into_iter()
                            .filter(|subscriber| {
                                let (s, r) = mpsc::channel();
                                let res = match (*subscriber)
                                    .send((topic.clone(), data.clone(), s)) {
                                    Ok(_) => {
                                        let _ = r.recv();
                                        true
                                    }
                                    // Remove senders that failed
                                    Err(mpsc::SendError(_)) => false,
                                };
                                res
                            })
                            .collect::<Vec<_>>();
                        self.subscribers.insert(topic.clone(), new_subscribers);
                        let _ = callback.send(());
                    }
                }
                Err(_) => (),
            }
        }
    }
}

/// PublisherAccessor is the gateway for Publisher
#[derive(Clone)]
pub struct PublisherAccessor<T: Sized + Clone> {
    sender: mpsc::Sender<PublisherMessage<T>>,
}

impl<T: Sized + Clone> PublisherAccessor<T> {
    fn new(sender: mpsc::Sender<PublisherMessage<T>>) -> Self {
        PublisherAccessor { sender: sender }
    }

    /// Subscribe to a topic
    pub fn subscribe(&self, topic: Topic, chan: SubscriberSender<T>) {
        let _ = self.sender.send(PublisherMessage::Subscribe(topic, chan));
    }

    pub fn send(&self, topic: Topic, data: T) {
        let (s, r) = mpsc::channel();
        let _ = self.sender.send(PublisherMessage::Send(topic, data, s));
        let _ = r.recv();
    }

    pub fn send_async(&self, topic: Topic, data: T) -> mpsc::Receiver<()> {
        let (s, r) = mpsc::channel();
        let _ = self.sender.send(PublisherMessage::Send(topic, data, s));
        r
    }

    /// Shutdown publisher
    pub fn shutdown(&self) {
        let _ = self.sender.send(PublisherMessage::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use pubsub::Publisher;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn subscribe_and_receive() {
        let mut publisher = Publisher::new();
        let accessor = publisher.accessor();
        let handle = thread::spawn(move || publisher.run());

        let (sender, receiver) = mpsc::channel();
        accessor.subscribe(Vec::from("test"), sender);
        let (sender1, receiver1) = mpsc::channel();
        accessor.subscribe(Vec::from("test1"), sender1);

        let accessor_ = accessor.clone();
        thread::spawn(move || accessor_.send(Vec::from("test"), "test"));
        let result = receiver.recv().unwrap();
        let _ = result.2.send(());
        assert_eq!((result.0, result.1), (Vec::from("test"), "test"));

        let accessor_ = accessor.clone();
        thread::spawn(move || accessor_.send(Vec::from("test1"), "test"));
        let result = receiver.recv_timeout(Duration::from_secs(1));
        assert!(result.is_err());

        let result = receiver1.recv().unwrap();
        let _ = result.2.send(());
        assert_eq!((result.0, result.1), (Vec::from("test1"), "test"));


        accessor.shutdown();
        let _ = handle.join();
    }

}
