// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Read, Write};

pub trait NonVolatileMemory : Read + Write {}

use memmap::{Mmap, MmapViewSync, Protection};

pub struct MmapedFile {
    // TODO: use or remove `size`
    #[allow(unused)]
    size: usize,
    offset: usize,
    mmap: Option<MmapViewSync>,
}

pub struct OutOfBoundsError;

use std::path::PathBuf;
use std::io;
use std::fs::OpenOptions;

impl MmapedFile {

    pub fn new(path: PathBuf, size: usize) -> Result<Self, io::Error> {
        let file = OpenOptions::new().create(true).write(true).open(path.as_path())?;
        let _ = file.set_len(size as u64)?;
        let mmap = Mmap::open_path(path.as_path(), Protection::ReadWrite)?;
        Ok(MmapedFile{
            size: size,
            offset: 0,
            mmap: Some(mmap.into_view_sync()),
        })
    }


    pub fn new_anonymous(size: usize) -> Result<Self, io::Error> {
        let mmap = Mmap::anonymous(size, Protection::ReadWrite)?;
        Ok(MmapedFile{
            size: size,
            offset: 0,
            mmap: Some(mmap.into_view_sync()),
        })
    }

    pub fn claim(&mut self, len: usize) -> Result<MmapedRegion, io::Error> {
        let (mut new_view, view) = ::std::mem::replace(&mut self.mmap, None).unwrap().split_at(self.offset + len)?;
        new_view.restrict(0, len)?;
        self.mmap = Some(view);
        self.offset += len;
        Ok(MmapedRegion {
            mmap: new_view,
            len: len,
        })
    }
}

pub struct MmapedRegion {
    mmap: MmapViewSync,
    // TODO: use or remove `len`
    #[allow(unused)]
    len: usize,
}

impl Read for MmapedRegion {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (unsafe { self.mmap.as_slice() }).read(buf)
    }
}

impl Write for MmapedRegion {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (unsafe { self.mmap.as_mut_slice() }).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.mmap.flush()
    }
}

impl<'a> NonVolatileMemory for MmapedRegion {}

