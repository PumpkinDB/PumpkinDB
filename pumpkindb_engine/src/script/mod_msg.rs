// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Env, EnvId, Module, PassResult, Error, ERROR_EMPTY_STACK, offset_by_size};
use super::super::messaging;

use std::marker::PhantomData;

instruction!(PUBLISH, (a, b => ), b"\x87PUBLISH");
instruction!(SUBSCRIBE, (a => ), b"\x89SUBSCRIBE");
instruction!(UNSUBSCRIBE, (a => ), b"\x8bUNSUBSCRIBE");

pub struct Handler<'a, P: messaging::Publisher, S: messaging::Subscriber> {
    publisher: P,
    subscriber: S,
    phantom: PhantomData<&'a ()>,
}

impl<'a, P: messaging::Publisher, S: messaging::Subscriber> Module<'a> for Handler<'a, P, S> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        try_instruction!(env, self.handle_publish(env, instruction, pid));
        try_instruction!(env, self.handle_subscribe(env, instruction, pid));
        try_instruction!(env, self.handle_unsubscribe(env, instruction, pid));
        Err(Error::UnknownInstruction)
    }
}

impl<'a, P: messaging::Publisher, S: messaging::Subscriber> Handler<'a, P, S> {
    pub fn new(publisher: P, subscriber: S) -> Self {
        Handler {
            publisher: publisher,
            subscriber: subscriber,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn handle_publish(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        instruction_is!(env, instruction, PUBLISH);
        let topic = stack_pop!(env);
        let data = stack_pop!(env);

        self.publisher.publish(topic, data);

        Ok(())
    }

    #[inline]
    fn handle_subscribe(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        instruction_is!(env, instruction, SUBSCRIBE);

        let topic = stack_pop!(env);

        match env.published_message_callback() {
            None => (),
            Some(cb) => {
                let ident = self.subscriber.subscribe(topic, cb);
                let slice = alloc_and_write!(&ident, env);
                env.push(slice);
            }
        }

        Ok(())
    }

    #[inline]
    fn handle_unsubscribe(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        instruction_is!(env, instruction, UNSUBSCRIBE);

        let identifier = stack_pop!(env);

        self.subscriber.unsubscribe(identifier);

        Ok(())
    }

}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use pumpkinscript::parse;
    use messaging;
    use script::{Env, Scheduler, Error, RequestMessage, ResponseMessage, EnvId};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use storage;
    use timestamp;

    use std::time::Duration;

    #[test]
    fn subscribe_publish() {
        let (sender0, receiver0) = mpsc::channel();
        eval!("\"Topic\" SUBSCRIBE \"Hello\" \"Topic\" PUBLISH",
              env,
              result,
              sender0.clone(), receiver0,
              {
                  assert!(!result.is_err());

                  let result = receiver0.recv_timeout(Duration::from_secs(1)).unwrap();
                  assert_eq!(result, (Vec::from("Topic"), Vec::from("Hello")));
              });

        eval!("\"Hello\" \"Topic1\" PUBLISH",
              env,
              result,
              sender0.clone(), receiver0,
              {
                  assert!(!result.is_err());
                  assert!(receiver0.recv_timeout(Duration::from_secs(1)).is_err());
              });

    }


    #[test]
    fn unsubscribe() {
        let (sender0, receiver0) = mpsc::channel();

        eval!("\"Topic\" SUBSCRIBE UNSUBSCRIBE \"Hello\" \"Topic\" PUBLISH",
              env,
              result,
              sender0.clone(), receiver0,
              {
                  assert!(!result.is_err());
                  assert!(receiver0.recv_timeout(Duration::from_secs(1)).is_err());
              });

    }


}
