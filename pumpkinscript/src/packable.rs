// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub trait Packable {
    fn pack(&self) -> Vec<u8>;
}

pub trait Unpackable<T>: Sized {
    fn unpack(&self) -> Option<T>;
}

impl Packable for f32 {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4);
        bytes.write_f32::<BigEndian>(*self).unwrap();
        if self.is_sign_negative() {
            for i in 0..4 {
                bytes[i] ^= 0xff;
            }
        } else {
            bytes[0] ^= 0x80;
        }
        bytes
    }
}

impl<'a> Unpackable<f32> for &'a [u8] {
    fn unpack(&self) -> Option<f32> {
        let mut vec: Vec<u8> = Vec::from(*self);
        
        if vec.len() != 4 {
            return None
        }
        if vec[0] >> 7 == 1u8 {
            vec[0] ^= 0x80;
        } else {
            for i in 0..4 {
                vec[i] ^= 0xff;
            }
        }
        if let Ok(f) = vec.as_slice().read_f32::<BigEndian>() {
            Some(f)
        } else {
            None
        }
    }
}


impl Packable for f64 {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        bytes.write_f64::<BigEndian>(*self).unwrap();
        if self.is_sign_negative() {
            for i in 0..8 {
                bytes[i] ^= 0xff;
            }
        } else {
            bytes[0] ^= 0x80;
        }
        bytes
    }
}

impl<'a> Unpackable<f64> for &'a [u8] {
    fn unpack(&self) -> Option<f64> {
        let mut vec: Vec<u8> = Vec::from(*self);
        
        if vec.len() != 8 {
            return None
        }
        if vec[0] >> 7 == 1u8 {
            vec[0] ^= 0x80;
        } else {
            for i in 0..8 {
                vec[i] ^= 0xff;
            }
        }
        if let Ok(f) = vec.as_slice().read_f64::<BigEndian>() {
            Some(f)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_pos() {
        let v = 5.0f32;
        assert_eq!(v, v.pack().as_slice().unpack().unwrap());
    }

    #[test]
    fn test_f32_neg() {
        let v = 5.0f32;
        assert_eq!(v, v.pack().as_slice().unpack().unwrap());
    }

    #[test]
    fn test_f64_pos() {
        let v = 5.0f64;
        assert_eq!(v, v.pack().as_slice().unpack().unwrap());
    }

    #[test]
    fn test_f64_neg() {
        let v = 5.0f64;
        assert_eq!(v, v.pack().as_slice().unpack().unwrap());
    }
}
