// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//////
//HLC 
//////

use hlc;
use std::sync::Mutex;

lazy_static! {

   static ref HLC_CLOCK: Mutex<hlc::Clock<hlc::Wall>> = Mutex::new(hlc::Clock::wall());

}

pub fn hlc() -> hlc::Timestamp<hlc::WallT> {
    (*HLC_CLOCK).lock().unwrap().now()
}



//////////////////////////////
//Atomic Counter Value (ACV)
//////////////////////////////
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

lazy_static! {
   //will init to ATOMIC_USIZE_INIT which is 0 
   static ref ACV_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
}

pub fn acv_count() -> usize {
    //increment by 1 but will return old value 
    ACV_COUNT.fetch_add(1, Ordering::SeqCst)
}


#[cfg(test)]
mod tests {

    use logicalstamp;

    #[test]
    fn test_hlc() {
        let t1 = logicalstamp::hlc();
        let t2 = logicalstamp::hlc();
        assert!(t2 > t1);
    }

    #[test]
    fn text_acv() {
        let c1 = logicalstamp::acv_count();
        let c2 = logicalstamp::acv_count();
        assert!(c2 > c1);
    }
}