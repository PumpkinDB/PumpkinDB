// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use hlc;
use std::sync::Mutex;

lazy_static! {

   static ref HLC_CLOCK: Mutex<hlc::Clock<hlc::Wall>> = Mutex::new(hlc::Clock::wall());

}

pub fn hlc() -> hlc::Timestamp<hlc::WallT> {
    (*HLC_CLOCK).lock().unwrap().now()
}

#[cfg(test)]
mod tests {

    use timestamp;

    #[test]
    fn test() {
        let t1 = timestamp::hlc();
        let t2 = timestamp::hlc();
        assert!(t2 > t1);
    }
}