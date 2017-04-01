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

#[cfg(not(feature = "static_module_dispatch"))]
macro_rules! for_each_dispatcher {
    ($module: ident, $dispatcher : expr, $expr: expr) => {
        for mut $module in $dispatcher.dispatchers.iter_mut() {
            $expr
        }
    };
}

#[cfg(feature = "static_module_dispatch")]
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

    }};
}

pub struct StandardDispatcher<'a> {
    #[cfg(not(feature = "static_module_dispatch"))]
    dispatchers: Vec<Box<Dispatcher<'a> + 'a>>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_core"))]
    core: mod_core::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_stack"))]
    stack: mod_stack::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_binaries"))]
    binaries: mod_binaries::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_numbers"))]
    numbers: mod_numbers::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_storage"))]
    storage: mod_storage::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_hash"))]
    hash: mod_hash::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_hlc"))]
    hlc: mod_hlc::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_json"))]
    json: mod_json::Handler<'a>,
    #[cfg(all(feature = "static_module_dispatch", feature = "mod_msg"))]
    msg: mod_msg::Handler<'a>,
}

impl<'a> StandardDispatcher<'a> {

    pub fn new<P: 'a, S: 'a>(db: &'a storage::Storage<'a>,
               publisher: P, subscriber: S,
               timestamp_state: Arc<timestamp::Timestamp>)
               -> Self where P : messaging::Publisher, S : messaging::Subscriber {
        #[cfg(not(feature="static_module_dispatch"))]
        {
            let mut mods : Vec<Box<Dispatcher<'a> + 'a>> = Vec::new();
            #[cfg(feature="mod_core")]
            mods.push(Box::new(mod_core::Handler::new()));
            #[cfg(feature="mod_stack")]
            mods.push(Box::new(mod_stack::Handler::new()));
            #[cfg(feature="mod_binaries")]
            mods.push(Box::new(mod_binaries::Handler::new()));
            #[cfg(feature="mod_numbers")]
            mods.push(Box::new(mod_numbers::Handler::new()));
            #[cfg(feature="mod_storage")]
            mods.push(Box::new(mod_storage::Handler::new(db)));
            #[cfg(feature="mod_hash")]
            mods.push(Box::new(mod_hash::Handler::new()));
            #[cfg(feature="mod_hlc")]
            mods.push(Box::new(mod_hlc::Handler::new(timestamp_state)));
            #[cfg(feature="mod_json")]
            mods.push(Box::new(mod_json::Handler::new()));
            #[cfg(feature="mod_msg")]
            mods.push(Box::new(mod_msg::Handler::new(publisher, subscriber)));
            return StandardDispatcher {
                dispatchers: mods,
            }
        }
        #[cfg(feature = "static_module_dispatch")]
        {
                return StandardDispatcher {
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
            }
        }
    }
}

impl<'a> Dispatcher<'a> for StandardDispatcher<'a> {
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