// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use hlc;
use std::sync::Mutex;
use memmap::{MmapViewSync, Mmap, Protection};

#[derive(Debug)]
pub struct Timestamp {
    clock: Mutex<(hlc::Clock<hlc::Wall>, MmapViewSync)>,
}

impl Timestamp {
    /// Create a new Timestamp clock. First the passed in memory map will be checked to check if
    /// a previous timestamp exists. If one exists (i.e. if the results aren't 20 bytes of 0) it
    /// will be "observed" by the HLC library.
    pub fn new(scratchpad: Option<MmapViewSync>) -> Self {
        if scratchpad.is_some() {
            let scratchpad = scratchpad.unwrap();
            let clock = {
                let mut clock = hlc::Clock::wall();
                let previous = unsafe { scratchpad.as_slice() };
                let res = hlc::Timestamp::read_bytes(previous);
                if res.is_ok() {
                    let res = res.unwrap();
                    if res > clock.now() {
                        let _ = clock.observe(&res);
                    }
                }
                clock
            };
            Timestamp {
                clock: Mutex::new((clock, scratchpad)),
            }
        } else {
            let clock = hlc::Clock::wall();
            let scratchpad = Mmap::anonymous(20, Protection::ReadWrite).unwrap();
            Timestamp {
                clock: Mutex::new((clock, scratchpad.into_view_sync()))
            }
        }
    }

    pub fn hlc(&self) -> hlc::Timestamp<hlc::WallT> {
        let mut clock = self.clock.lock().unwrap();
        let now = clock.0.now();
        let ref mut state = clock.1;
        let _ = unsafe { &now.write_bytes(&mut state.as_mut_slice()).unwrap() };
        now
    }

}