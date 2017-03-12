// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use hlc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Timestamp {
    clock: Mutex<hlc::Clock<hlc::Wall>>
}

impl Timestamp {
    pub fn new() -> Self {
        Timestamp {
            clock: Mutex::new(hlc::Clock::wall())
        }
    }

    pub fn hlc(&self) -> hlc::Timestamp<hlc::WallT> {
        self.clock.lock().unwrap().now()
    }

}