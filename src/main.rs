// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![cfg_attr(test, feature(test))]

#![feature(alloc, heap_api)]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate matches;

#[cfg(test)]
extern crate test;

// Parser
#[macro_use]
extern crate nom;

extern crate multiqueue;
extern crate snowflake;

pub mod script;

fn main() {}
