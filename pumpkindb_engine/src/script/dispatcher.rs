// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;
use std::iter::Iterator;

pub trait Dispatcher<'a> {
    #[allow(unused_variables)]
    fn init(&mut self, env: &mut Env<'a>, pid: EnvId) {}
    #[allow(unused_variables)]
    fn done(&mut self, env: &mut Env<'a>, pid: EnvId) {}
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a>;
}

include!("macros.rs");

impl<'a> Dispatcher<'a> for Vec<Box<Dispatcher<'a>>> {
    fn init(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for mut disp in self.into_iter() {
            disp.init(env, pid);
        }
    }
    fn done(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for mut disp in self.into_iter() {
            disp.done(env, pid);
        }
    }
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        let mut iter = self.into_iter();
        loop {
            match iter.next() {
                None => break,
                Some(mut disp) => {
                    let result = disp.handle(env, instruction, pid);
                    if result.is_unhandled() {
                        continue
                    }
                    return result;
                }
            }
        }
        Err(Error::UnknownInstruction)
    }
}

macro_rules! for_each_dispatcher {
    ($module: ident, $dispatcher : expr, $expr: expr) => {{
        #[cfg(feature="mod_core")]
        {
           let ref mut $module = $dispatcher.core;
           $expr
        }
        #[cfg(feature="mod_stack")]
        {
           let ref mut $module = $dispatcher.stack;
           $expr
        }
        #[cfg(feature="mod_queue")]
        {
           let ref mut $module = $dispatcher.queue;
           $expr
        }
        #[cfg(feature="mod_binaries")]
        {
           let ref mut $module = $dispatcher.binaries;
           $expr
        }
        #[cfg(feature="mod_numbers")]
        {
           let ref mut $module = $dispatcher.numbers;
           $expr
        }
        #[cfg(feature="mod_storage")]
        {
           let ref mut $module = $dispatcher.storage;
           $expr
        }
        #[cfg(feature="mod_hash")]
        {
           let ref mut $module = $dispatcher.hash;
           $expr
        }
        #[cfg(feature="mod_hlc")]
        {
           let ref mut $module = $dispatcher.hlc;
           $expr
        }
        #[cfg(feature="mod_json")]
        {
           let ref mut $module = $dispatcher.json;
           $expr
        }
        #[cfg(feature="mod_msg")]
        {
           let ref mut $module = $dispatcher.msg;
           $expr
        }
        #[cfg(feature="mod_uuid")]
        {
            let ref mut $module = $dispatcher.uuid;
            $expr
        }
        #[cfg(feature="mod_string")]
        {
            let ref mut $module = $dispatcher.string;
            $expr
        }
    }};
}

use super::super::nvmem::NonVolatileMemory;

pub struct StandardDispatcher<'a, P: 'a, S: 'a, N: 'a, T>
    where P : messaging::Publisher, S : messaging::Subscriber,
          N : NonVolatileMemory, T : AsRef<storage::Storage<'a>> + 'a
{
    #[cfg(feature = "mod_core")]
    core: mod_core::Handler<'a>,
    #[cfg(feature = "mod_stack")]
    stack: mod_stack::Handler<'a>,
    #[cfg(feature = "mod_queue")]
    queue: mod_queue::Handler<'a>,
    #[cfg(feature = "mod_binaries")]
    binaries: mod_binaries::Handler<'a>,
    #[cfg(feature = "mod_numbers")]
    numbers: mod_numbers::Handler<'a>,
    #[cfg(feature = "mod_storage")]
    storage: mod_storage::Handler<'a, T, N>,
    #[cfg(feature = "mod_hash")]
    hash: mod_hash::Handler<'a>,
    #[cfg(feature = "mod_hlc")]
    hlc: mod_hlc::Handler<'a, N>,
    #[cfg(feature = "mod_json")]
    json: mod_json::Handler<'a>,
    #[cfg(feature = "mod_msg")]
    msg: mod_msg::Handler<'a, P, S>,
    #[cfg(feature = "mod_uuid")]
    uuid: mod_uuid::Handler<'a>,
    #[cfg(feature = "mod_string")]
    string: mod_string::Handler<'a>
}


impl<'a, P: 'a, S: 'a, N: 'a, T> StandardDispatcher<'a, P, S, N, T>
    where P : messaging::Publisher, S : messaging::Subscriber,
          N : NonVolatileMemory, T : AsRef<storage::Storage<'a>> + 'a {

    pub fn new(db: T,
               publisher: P, subscriber: S,
               timestamp_state: Arc<timestamp::Timestamp<N>>)
               -> Self {
        StandardDispatcher {
                #[cfg(feature = "mod_core")]
                    core: mod_core::Handler::new(),
                #[cfg(feature = "mod_stack")]
                    stack: mod_stack::Handler::new(),
               #[cfg(feature = "mod_stack")]
                    queue: mod_queue::Handler::new(),
                #[cfg(feature = "mod_binaries")]
                    binaries: mod_binaries::Handler::new(),
                #[cfg(feature = "mod_numbers")]
                    numbers: mod_numbers::Handler::new(),
                #[cfg(feature = "mod_storage")]
                    storage: mod_storage::Handler::new(db, timestamp_state.clone()),
                #[cfg(feature = "mod_hash")]
                    hash: mod_hash::Handler::new(),
                #[cfg(feature = "mod_hlc")]
                    hlc: mod_hlc::Handler::new(timestamp_state),
                #[cfg(feature = "mod_json")]
                    json: mod_json::Handler::new(),
                #[cfg(feature = "mod_msg")]
                    msg: mod_msg::Handler::new(publisher, subscriber),
                #[cfg(feature = "mod_uuid")]
                    uuid: mod_uuid::Handler::new(),
                #[cfg(feature = "mod_string")]
                    string: mod_string::Handler::new(),
        }
    }
}

impl<'a, P: 'a, S: 'a, N: 'a, T> Dispatcher<'a> for StandardDispatcher<'a, P, S, N, T>
    where P : messaging::Publisher, S : messaging::Subscriber, N : NonVolatileMemory,
          T : AsRef<storage::Storage<'a>> + 'a {
    fn init(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for_each_dispatcher!(disp, self, disp.init(env, pid));
    }
    fn done(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for_each_dispatcher!(disp, self, disp.done(env, pid));
    }
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        for_each_dispatcher!(disp, self, {
           let result = disp.handle(env, instruction, pid);
           if !result.is_unhandled() {
              return result;
           }
        });
        Err(Error::UnknownInstruction)
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

  use pumpkinscript::parse;
  use script::{Env, EnvId, PassResult,
               Scheduler, SchedulerHandle, Error, RequestMessage, ResponseMessage,
               Dispatcher, TryInstruction};
  use std::sync::mpsc;
  use crossbeam;

  use std::marker::PhantomData;

  struct MyDispatcher<'a> {
      phantom: PhantomData<&'a ()>,
  }

  impl<'a> MyDispatcher<'a> {

      pub fn new() -> Self {
          MyDispatcher{ phantom: PhantomData }
      }

      pub fn handle_test(&mut self, env: &mut Env<'a>,
                          instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
          return_unless_instructions_equal!(instruction, b"\x84TEST");
          env.push(b"TEST");
          Ok(())
      }

  }

  impl<'a> Dispatcher<'a> for MyDispatcher<'a> {
     fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
         self.handle_test(env, instruction, pid)
             .if_unhandled_try(|| Err(Error::UnknownInstruction))
     }
  }

  #[test]
  pub fn dynamic_dispatch() {
      crossbeam::scope(|scope| {
          let dispatchers: Vec<Box<Dispatcher>> = vec![Box::new(MyDispatcher::new())];
          let (mut scheduler, sender) = Scheduler::new(dispatchers);
          let handle = scope.spawn(move || scheduler.run() );
          let sender_ = sender.clone();
          let script = parse("TEST").unwrap();
          let (callback, receiver) = mpsc::channel::<ResponseMessage>();
          let (sender0, _) = mpsc::channel();
          sender.schedule_env(EnvId::new(), script.clone(), callback, Box::new(sender0));
          match receiver.recv() {
              Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                  // terminated without an error
                  let mut stack_ = Vec::with_capacity(stack.len());
                  for i in 0..(&stack).len() {
                      stack_.push((&stack[i]).as_slice());
                  }
                  let mut script_env = Env::new_with_stack(stack_).unwrap();
                  let val = script_env.pop().unwrap();
                  assert_eq!(val, b"TEST");
              },
              Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                  let _ = sender.send(RequestMessage::Shutdown);
                  panic!("error: {:?}", err);
              }
              Err(err) => {
                  let _ = sender.send(RequestMessage::Shutdown);
                  panic!("recv error: {:?}", err);
             }
         }
         sender_.shutdown();
         let _ = handle.join();
    });
  }

}
