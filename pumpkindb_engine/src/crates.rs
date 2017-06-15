// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate matches;

#[cfg(test)]
extern crate test;

extern crate core;

extern crate num_bigint;
extern crate num_iter;
extern crate num_traits;
extern crate snowflake;
pub extern crate lmdb_zero as lmdb;
#[cfg(test)]
extern crate tempdir;
#[cfg(test)]
extern crate crossbeam;

extern crate libc;

extern crate hybrid_clocks as hlc;

extern crate byteorder;

extern crate config;

#[macro_use]
extern crate lazy_static;

extern crate crypto;

extern crate serde_json;
extern crate serde_cbor;

extern crate rand;

#[macro_use]
extern crate log;

extern crate memmap;

#[macro_use]
extern crate pumpkinscript;

extern crate uuid;

extern crate num_cpus;