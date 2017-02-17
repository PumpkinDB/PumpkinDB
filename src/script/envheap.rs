// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
///
/// # Env Heap
///
/// This module implements algorithms for managing Env heap
///

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
        let (mut ptr, chunk) = self.chunks.pop().unwrap();
        let cap = chunk.cap();
        if ptr + size > cap {
            self.chunks.push((ptr, chunk));
            self.chunks.push((0, RawVec::with_capacity(cmp::max(cap, size))));
            self.alloc(size)
        } else {
            let slice_ptr = unsafe { chunk.ptr().offset(ptr as isize) };
            ptr += size;
            self.chunks.push((ptr, chunk));
            unsafe { slice::from_raw_parts_mut(slice_ptr, size) }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {
    use script::envheap::EnvHeap;

    #[test]
    fn alloc() {
        let mut heap = EnvHeap::new(32_768);
        let sz = 20_000;
        for i in 1..100 {
            heap.alloc(sz);
        }
    }
}