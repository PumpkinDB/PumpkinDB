// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

macro_rules! write_size_into_slice {
    ($size:expr, $slice: expr) => {
     match $size {
        0...120 => {
            $slice[0] = $size as u8;
            1
        }
        121...255 => {
            $slice[0] = 121u8;
            $slice[1] = $size as u8;
            2
        }
        256...65535 => {
            $slice[0] = 122u8;
            $slice[1] = ($size >> 8) as u8;
            $slice[2] = $size as u8;
            3
        }
        65536...4294967296 => {
            $slice[0] = 123u8;
            $slice[1] = ($size >> 24) as u8;
            $slice[2] = ($size >> 16) as u8;
            $slice[3] = ($size >> 8) as u8;
            $slice[4] = $size as u8;
            5
        }
        _ => unreachable!(),
    }
    };
}

macro_rules! data {
    ($ptr:expr) => {
        {
          let (_, size) = binparser::data_size($ptr).unwrap();
          (&$ptr[offset_by_size(size)..$ptr.len()], size)
        }
    };
}

macro_rules! handle_words {
    ($me: expr, $env: expr, $program: expr, $word: expr, $res: ident,
     $pid: ident, [ $($name: ident),* ], $block: expr) => {
    {
      let mut env = $env;
      $(
       env =
        match $me.$name(env, $word, $pid) {
          Err((env, Error::Reschedule)) => return Ok((env, Some($program.clone()))),
          Err((env, Error::UnknownWord)) => env,
          Err((env, err)) => return Err((env, err)),
          Ok($res) => $block
        };
      )*
      return Err((env, Error::UnknownWord))
    }
    };
}

macro_rules! validate_lockout {
    ($env: expr, $name: expr, $pid: expr) => {
        if let Some((pid_, _)) = $name {
            if pid_ != $pid {
                return Err(($env, Error::Reschedule))
            }
        }
    };
}

macro_rules! read_or_write_transaction {
    ($me: expr, $env: expr) => {
        if let Some((_, ref txn)) = $me.db_write_txn {
            txn.deref()
        } else if let Some((_, ref txn)) = $me.db_read_txn {
            txn.deref()
        } else {
            return Err(($env, Error::NoTransaction));
        };
    };
}