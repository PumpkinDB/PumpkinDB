// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::Error;
use super::envheap::EnvHeap;
use super::super::messaging;

use std::collections::BTreeMap;

/// Initial stack size
pub const STACK_SIZE: usize = 32_768;
/// Initial heap size
pub const HEAP_SIZE: usize = 32_768;

/// Env is a representation of a stack and the heap.
///
/// Doesn't need to be used directly as it's primarily
/// used by [`Scheduler`](struct.Scheduler.html)
pub struct Env<'a> {
    pub program: Vec<&'a [u8]>,
    stack: Vec<&'a [u8]>,
    pub stack_size: usize,
    heap: EnvHeap,
    #[cfg(feature = "scoped_dictionary")]
    pub dictionary: Vec<BTreeMap<&'a [u8], &'a [u8]>>,
    #[cfg(not(feature = "scoped_dictionary"))]
    pub dictionary: BTreeMap<&'a [u8], &'a [u8]>,
    // current TRY status
    pub tracking_errors: usize,
    pub aborting_try: Vec<Error>,
    published_message_callback: Option<Box<messaging::PublishedMessageCallback + Send>>,
}

impl<'a> ::std::fmt::Debug for Env<'a> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        fmt.write_str("Env()")
    }
}

unsafe impl<'a> Send for Env<'a> {}

const _EMPTY: &'static [u8] = b"";

use std::mem;

impl<'a> Env<'a> {
    /// Creates an environment with [an empty stack of default size](constant.STACK_SIZE.html)
    pub fn new() -> Result<Self, Error> {
        Env::new_with_stack_size(STACK_SIZE)
    }

    /// Creates an environment with an empty stack of specific size
    pub fn new_with_stack_size(size: usize) -> Result<Self, Error> {
        Env::new_with_stack(vec![_EMPTY; size], 0)
    }

    /// Creates an environment with an existing stack and a pointer to the
    /// topmost element (stack_size)
    ///
    /// This function is useful for working with result stacks received from
    /// [Scheduler](struct.Scheduler.html)
    pub fn new_with_stack(stack: Vec<&'a [u8]>, stack_size: usize) -> Result<Self, Error> {
        #[cfg(feature = "scoped_dictionary")]
        let dictionary = vec![BTreeMap::new()];
        #[cfg(not(feature = "scoped_dictionary"))]
        let dictionary = BTreeMap::new();
        Ok(Env {
            program: vec![],
            stack: stack,
            stack_size: stack_size,
            heap: EnvHeap::new(HEAP_SIZE),
            dictionary: dictionary,
            tracking_errors: 0,
            aborting_try: Vec::new(),
            published_message_callback: None,
        })
    }

    /// Returns the entire stack
    #[inline]
    pub fn stack(&self) -> &[&'a [u8]] {
        &self.stack.as_slice()[0..self.stack_size as usize]
    }

    /// Returns a copy of the entire stack
    #[inline]
    pub fn stack_copy(self) -> Vec<Vec<u8>> {
        self.stack[0..self.stack_size as usize].into_iter().map(|v| Vec::from(*v)).collect()
    }

    /// Returns top of the stack without removing it
    #[inline]
    pub fn stack_top(&self) -> Option<&'a [u8]> {
        if self.stack_size == 0 {
            None
        } else {
            Some(self.stack.as_slice()[self.stack_size as usize - 1])
        }
    }

    /// Removes the top of the stack and returns it
    #[inline]
    pub fn pop(&mut self) -> Option<&'a [u8]> {
        if self.stack_size == 0 {
            None
        } else {
            let val = Some(self.stack.as_slice()[self.stack_size as usize - 1]);
            self.stack.as_mut_slice()[self.stack_size as usize - 1] = _EMPTY;
            self.stack_size -= 1;
            val
        }
    }

    /// Pushes value on top of the stack
    #[inline]
    pub fn push(&mut self, data: &'a [u8]) {
        // check if we are at capacity
        if self.stack_size == self.stack.len() {
            let mut vec = vec![_EMPTY; STACK_SIZE];
            self.stack.append(&mut vec);
        }
        self.stack.as_mut_slice()[self.stack_size] = data;
        self.stack_size += 1;
    }

    /// Allocates a slice off the Env-specific heap. Will be collected
    /// once this Env is dropped.
    pub fn alloc(&mut self, len: usize) -> Result<&'a mut [u8], Error> {
        Ok(unsafe { mem::transmute::<&mut [u8], &'a mut [u8]>(self.heap.alloc(len)) })
    }


    #[cfg(feature = "scoped_dictionary")]
    pub fn push_dictionary(&mut self) {
        let dict = self.dictionary.pop().unwrap();
        let new_dict = dict.clone();
        self.dictionary.push(dict);
        self.dictionary.push(new_dict);
    }

    #[cfg(feature = "scoped_dictionary")]
    pub fn pop_dictionary(&mut self) {
        self.dictionary.pop();
        if self.dictionary.len() == 0 {
            self.dictionary.push(BTreeMap::new());
        }
    }

    pub fn set_published_message_callback(&mut self,
                                          callback: Box<messaging::PublishedMessageCallback + Send>) {
        self.published_message_callback = Some(callback);
    }

    pub fn published_message_callback(&self) -> Option<Box<messaging::PublishedMessageCallback + Send>> {
        match self.published_message_callback {
            None => None,
            Some(ref cb) => Some(cb.cloned())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{Env, _EMPTY};

    #[test]
    fn env_stack_growth() {
        let mut env = Env::new().unwrap();
        let target = env.stack.len() * 100;
        for _ in 1..target {
            env.push(_EMPTY);
        }
        assert!(env.stack.len() >= target);
    }

}
