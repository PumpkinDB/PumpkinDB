// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns, advanced_slice_patterns)]

#![cfg_attr(test, feature(test))]
#![cfg_attr(not(target_os = "windows"), feature(alloc))]
#![feature(alloc)]

include!("crates.rs");

pub mod script;
pub mod messaging;
pub mod storage;
pub mod timestamp;
pub mod nvmem;
