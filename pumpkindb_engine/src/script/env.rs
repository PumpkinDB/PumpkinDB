// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::Error;
use super::envheap::EnvHeap;
use super::super::messaging;

use std::collections::BTreeMap;

/// Initial heap size
pub const HEAP_SIZE: usize = 32_768;

use std::collections::VecDeque;
/// Env is a representation of a stack and the heap.
///
/// Doesn't need to be used directly as it's primarily
/// used by [`Scheduler`](struct.Scheduler.html)
pub struct Env<'a> {
    pub program: Vec<&'a [u8]>,
    stack: VecDeque<Vec<&'a [u8]>>,
    return_stack: Vec<&'a [u8]>,
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
    /// Creates an environment with an empty stack
    pub fn new() -> Result<Self, Error> {
        Env::new_with_stack(vec![])
    }

    /// Creates an environment with an existing stack and a pointer to the
    /// topmost element (stack_size)
    ///
    /// This function is useful for working with result stacks received from
    /// [Scheduler](struct.Scheduler.html)
    pub fn new_with_stack(stack: Vec<&'a [u8]>) -> Result<Self, Error> {
        #[cfg(feature = "scoped_dictionary")]
        let dictionary = vec![BTreeMap::new()];
        #[cfg(not(feature = "scoped_dictionary"))]
        let dictionary = BTreeMap::new();
        let mut stacks = VecDeque::new();
        stacks.push_front(stack);
        Ok(Env {
            program: vec![],
            stack: stacks,
            return_stack: vec![],
            heap: EnvHeap::new(HEAP_SIZE),
            dictionary: dictionary,
            tracking_errors: 0,
            aborting_try: Vec::new(),
            published_message_callback: None,
        })
    }

    #[inline]
    pub fn push_stack(&mut self) {
        self.stack.push_front(vec![]);
    }

    #[inline]
    pub fn pop_stack(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop_front();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn push_return(&mut self, data: &'a [u8]) {
        self.return_stack.push(data);
    }

    #[inline]
    pub fn pop_return(&mut self) -> Option<&'a [u8]> {
        self.return_stack.pop()
    }

    /// Returns the entire stack
    #[inline]
    pub fn stack(&self) -> &[&'a [u8]] {
        &self.stack.front().unwrap()
    }

    /// Returns a copy of the entire stack
    #[inline]
    pub fn stack_copy(&self) -> Vec<Vec<u8>> {
        self.stack.front().unwrap().into_iter().map(|v| Vec::from(*v)).collect()
    }

    /// Returns top of the stack without removing it
    #[inline]
    pub fn stack_top(&self) -> Option<&'a [u8]> {
        let s = self.stack.front().unwrap();
        let len = s.len();
        if len == 0 {
            None
        } else {
            Some(&s[len - 1])
        }
    }

    /// Removes the top of the stack and returns it
    #[inline]
    pub fn pop(&mut self) -> Option<&'a [u8]> {
        self.stack.front_mut().unwrap().pop()
    }

    /// Pushes value on top of the stack
    #[inline]
    pub fn push(&mut self, data: &'a [u8]) {
        self.stack.front_mut().unwrap().push(data);
    }

    /// Allocates a slice off the Env-specific heap. Will be collected
    /// once this Env is dropped.
    pub fn alloc(&mut self, len: usize) -> Result<&'a mut [u8], Error> {
        Ok(unsafe { mem::transmute::<&mut [u8], &'a mut [u8]>(self.heap.alloc(len)) })
    }


    #[cfg(feature = "scoped_dictionary")]
    pub fn push_dictionary(&mut self) {
        self.dictionary.push(BTreeMap::new());
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