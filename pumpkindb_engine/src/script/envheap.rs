// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//!
//! # Env Heap
//!
//! This module implements algorithms for managing Env heap
//!

use alloc::raw_vec::RawVec;
use std::cmp;
use std::slice;

/// EnvHeap's main goal is to ensure that raw chunks
/// of memory are never reallocated, thus invalidating
/// references.
///
/// EnvHeap accomplishes that by adding new chunks instead
/// of resizing existing ones.
pub struct EnvHeap {
    chunks: Vec<(usize, RawVec<u8>)>,
}

impl EnvHeap {
    /// Creates new EnvHeap with a certain chunk size, which
    /// can't be changed later
    pub fn new(chunk_size: usize) -> Self {
        EnvHeap { chunks: vec![(0, RawVec::with_capacity(chunk_size))] }
    }

    /// Allocates a new mutable slice
    pub fn alloc(&mut self, size: usize) -> &mut [u8] {
        let nchunks = self.chunks.len();
        //Look for chunks with enough free space.
        for i in 0..nchunks {
            let cap = self.chunks[i].1.cap();
            let ptr = self.chunks[i].0;
            if ptr + size > cap {
                if i == (nchunks - 1) {
                    self.chunks.push((0, RawVec::with_capacity(cmp::max(cap, size))));
                    return self.alloc(size)
                } else {
                    continue;
                }
            } else {
                let (mut ptr, chunk) = self.chunks.pop().unwrap();
                let slice_ptr = unsafe { chunk.ptr().offset(ptr as isize) };
                ptr += size;
                self.chunks.push((ptr, chunk));
                return unsafe { slice::from_raw_parts_mut(slice_ptr, size) }
            }
        }
        unreachable!();
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {
    use script::envheap::EnvHeap;

    #[test]
    fn alloc() {
        let mut heap = EnvHeap::new(32_768);
        let sz_0 = 20_000;
        let sz_1 = 10_000;
        for i in 1..100 {
            if (i % 2) == 0 {
                heap.alloc(sz_1);
            } else {
                heap.alloc(sz_0);
            }
        }
        assert_eq!(50, heap.chunks.len());
    }
}
