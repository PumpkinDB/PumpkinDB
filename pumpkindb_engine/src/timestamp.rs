// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use hlc;
use std::sync::Mutex;
use super::nvmem::NonVolatileMemory;

#[derive(Debug)]
pub struct Timestamp<N : NonVolatileMemory> {
    clock: Mutex<(hlc::Clock<hlc::Wall>, N)>,
}

impl<N : NonVolatileMemory> Timestamp<N> {
    /// Create a new Timestamp clock. First the passed in memory map will be checked to check if
    /// a previous timestamp exists. If one exists (i.e. if the results aren't 20 bytes of 0) it
    /// will be "observed" by the HLC library.
    pub fn new(mut nvmem: N) -> Self {
        let clock = {
            let mut clock = hlc::Clock::wall();
            let res = hlc::Timestamp::read_bytes(&mut nvmem);
            if res.is_ok() {
                let res = res.unwrap();
                if res > clock.now() {
                    let _ = clock.observe(&res);
                }
            }
            clock
        };
        Timestamp { clock: Mutex::new((clock, nvmem)) }
    }

    pub fn hlc(&self) -> hlc::Timestamp<hlc::WallT> {
        let mut clock = self.clock.lock().unwrap();
        let now = clock.0.now();
        let _ = &now.write_bytes(&mut clock.1).unwrap();
        now
    }

    pub fn observe(&self, other_time: &hlc::Timestamp<hlc::WallT>) -> Result<(), ()> {
        let mut clock = self.clock.lock().unwrap();
        match clock.0.observe(&other_time) {
            Ok(_) => {
                let _ = clock.0.now().write_bytes(&mut clock.1).unwrap();
                Ok(())
            },
            Err(_) => Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use nvmem::{MmapedFile};
    use timestamp::Timestamp;
    use hlc;

    #[test]
    fn order_guaranteed() {
        let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
        let region = nvmem.claim(20).unwrap();
        let timestamp = Timestamp::new(region);
        let p1 = timestamp.hlc();
        let p2 = timestamp.hlc();
        assert!(p2 > p1);
    }

    #[test]
    fn order_guaranteed_if_shifted() {
        // Creates a clock and moves it into the future. That timestamp get serialized to an mmap
        // which a Timestamp is created with, making the "future" value the last known value of the
        // timestamp. Two stamps are then taken, and we make sure that even with a timestamp "from
        // the future" as being the last known timestamp, the invariant that HLC moves forward (and
        // is ordered that way) holds.
        let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
        let mut clock = hlc::Clock::wall();
        clock.set_epoch(u32::max_value());
        let now = clock.now();
        let mut region = nvmem.claim(20).unwrap();
        let _ = &now.write_bytes(&mut region).unwrap();
        let timestamp = Timestamp::new(region);
        let p1 = timestamp.hlc();
        let p2 = timestamp.hlc();
        assert!(p1 < p2);
    }

    
    #[test]
    fn observe_updates_hlc() {
        let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
        let region = nvmem.claim(20).unwrap();
        let timestamp = Timestamp::new(region);
        let t0 = timestamp.hlc();
        let mut wall_clock = hlc::Clock::wall();
        let wall_epoch = t0.epoch + 1;
        wall_clock.set_epoch(wall_epoch);
        timestamp.observe(&wall_clock.now()).unwrap();

        let t1 = timestamp.hlc();

        assert_eq!(t1.epoch, wall_epoch);
    }

    use test::Bencher;

    #[bench]
    fn timestamp_generation(b: &mut Bencher) {
        let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
        let region = nvmem.claim(20).unwrap();
        let timestamp = Timestamp::new(region);
        b.iter(|| timestamp.hlc());
    }
}
