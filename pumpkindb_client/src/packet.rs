// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write, Read};
use std::io;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct PacketWriter<T : Write>(T);

impl<T : Write> PacketWriter<T> {
    pub fn new(writer: T) -> Self {
        PacketWriter(writer)
    }
    pub fn writer(self) -> T {
        self.0
    }
}

impl<T : Write> Write for PacketWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0.write_u32::<BigEndian>(buf.len() as u32) {
            Ok(()) => {
               match self.0.write(buf) {
                   Ok(sz) => Ok(sz + 4),
                   Err(err) => Err(err),
               }
            },
            Err(err) => Err(err),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub struct PacketReader<T : Read>(T);

impl<T : Read> PacketReader<T> {
    pub fn new(reader: T) -> Self {
        PacketReader(reader)
    }
    pub fn reader(self) -> T {
        self.0
    }

    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        match self.0.read_u32::<BigEndian>() {
            Ok(size) => {
                let mut buf = Vec::with_capacity(size as usize);
                unsafe { buf.set_len(size as usize); }
                match self.0.read(&mut buf) {
                    Ok(_) => {
                        Ok(buf)
                    },
                    Err(err) => Err(err),
                }
            },
            Err(err) => Err(err),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{PacketWriter, PacketReader};
    use std::io::{Write, Cursor};

    #[test]
    fn write() {
        let v = vec![];
        let mut w = PacketWriter::new(v);
        let _ = w.write("hello".as_bytes()).unwrap();
        let result = w.writer();
        assert_eq!(result, vec![0,0,0,5,b'h',b'e',b'l',b'l',b'o']);
    }

    #[test]
    fn read() {
        let v = vec![0,0,0,5,b'h',b'e',b'l',b'l',b'o'];
        let mut r = PacketReader::new(Cursor::new(v));
        let result = r.read().unwrap();
        assert_eq!(result, "hello".as_bytes());
    }
}
