// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::*;

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
        for mut disp in self.into_iter() {
            try_instruction!(env, disp.handle(env, instruction, pid));
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
    }};
}

pub struct StandardDispatcher<'a, P: 'a, S: 'a>
    where P : messaging::Publisher, S : messaging::Subscriber
{
    #[cfg(feature = "mod_core")]
    core: mod_core::Handler<'a>,
    #[cfg(feature = "mod_stack")]
    stack: mod_stack::Handler<'a>,
    #[cfg(feature = "mod_binaries")]
    binaries: mod_binaries::Handler<'a>,
    #[cfg(feature = "mod_numbers")]
    numbers: mod_numbers::Handler<'a>,
    #[cfg(feature = "mod_storage")]
    storage: mod_storage::Handler<'a>,
    #[cfg(feature = "mod_hash")]
    hash: mod_hash::Handler<'a>,
    #[cfg(feature = "mod_hlc")]
    hlc: mod_hlc::Handler<'a>,
    #[cfg(feature = "mod_json")]
    json: mod_json::Handler<'a>,
    #[cfg(feature = "mod_msg")]
    msg: mod_msg::Handler<'a, P, S>,
    #[cfg(feature = "mod_uuid")]
    uuid: mod_uuid::Handler<'a>,
}

impl<'a, P: 'a, S: 'a> StandardDispatcher<'a, P, S>
    where P : messaging::Publisher, S : messaging::Subscriber {

    pub fn new(db: &'a storage::Storage<'a>,
               publisher: P, subscriber: S,
               timestamp_state: Arc<timestamp::Timestamp>)
               -> Self {
        StandardDispatcher {
                #[cfg(feature = "mod_core")]
                    core: mod_core::Handler::new(),
                #[cfg(feature = "mod_stack")]
                    stack: mod_stack::Handler::new(),
                #[cfg(feature = "mod_binaries")]
                    binaries: mod_binaries::Handler::new(),
                #[cfg(feature = "mod_numbers")]
                    numbers: mod_numbers::Handler::new(),
                #[cfg(feature = "mod_storage")]
                    storage: mod_storage::Handler::new(db),
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
        }
    }
}

impl<'a, P: 'a, S: 'a> Dispatcher<'a> for StandardDispatcher<'a, P, S>
    where P : messaging::Publisher, S : messaging::Subscriber {
    fn init(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for_each_dispatcher!(disp, self, disp.init(env, pid));
    }
    fn done(&mut self, env: &mut Env<'a>, pid: EnvId) {
        for_each_dispatcher!(disp, self, disp.done(env, pid));
    }
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        for_each_dispatcher!(disp, self, try_instruction!(env, disp.handle(env, instruction, pid)));
        Err(Error::UnknownInstruction)
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

  use pumpkinscript::parse;
  use script::{Env, EnvId, PassResult,
               Scheduler, Error, RequestMessage, ResponseMessage,
               Dispatcher};
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
          instruction_is!(env, instruction, b"\x84TEST");
          env.push(b"TEST");
          Ok(())
      }

  }

  impl<'a> Dispatcher<'a> for MyDispatcher<'a> {
     fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
         try_instruction!(env, self.handle_test(env, instruction, pid));
         Err(Error::UnknownInstruction)
     }
  }

  #[test]
  pub fn dynamic_dispatch() {
      crossbeam::scope(|scope| {
          let (sender, receiver) = Scheduler::<Vec<Box<Dispatcher>>>::create_sender();
          let handle = scope.spawn(move || {
              let dispatchers: Vec<Box<Dispatcher>> = vec![Box::new(MyDispatcher::new())];
              let mut scheduler = Scheduler::new(dispatchers, receiver);
              scheduler.run()
          });
          let sender_ = sender.clone();
          let script = parse("TEST").unwrap();
          let (callback, receiver) = mpsc::channel::<ResponseMessage>();
          let (sender0, _) = mpsc::channel();
          let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(), script.clone(),
                                                          callback, Box::new(sender0)));
          match receiver.recv() {
              Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                  // terminated without an error
                  let mut stack_ = Vec::with_capacity(stack.len());
                  for i in 0..(&stack).len() {
                      stack_.push((&stack[i]).as_slice());
                  }
                  let mut script_env = Env::new_with_stack(stack_, stack_size).unwrap();
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
         let _ = sender_.send(RequestMessage::Shutdown);
         let _ = handle.join();
    });
  }

}
