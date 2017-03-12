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

macro_rules! handle_error {
    ($env: expr, $err: expr) => {
       handle_error!($env, $err, Ok(()))
    };
    ($env: expr, $err: expr, $body: expr) => {{
        if $env.tracking_errors > 0 {
            $env.aborting_try.push($err);
            $body
        } else {
            return Err($err)
        }
    }};
}

macro_rules! try_word {
  ($env: expr, $handler : expr) => {
    match $handler {
        Err(Error::UnknownWord) => (),
        Err(err @ Error::ProgramError(_)) => return handle_error!($env, err),
        Err(err) => return Err(err),
        Ok(()) => return Ok(())
    }
  };
}

macro_rules! stack_pop {
    ($env: expr) => {
        match $env.pop() {
            None => {
                return Err(error_empty_stack!())
            }
            Some(e) => {
                e
            }
        }
    }
}

macro_rules! word_is {
    ($env: expr, $word: expr, $exp: expr) => {
        if $word != $exp {
            return Err(Error::UnknownWord)
        }
    };
}

macro_rules! error_program {
    ($desc: expr, $details: expr, $code: expr) => {{
        let mut error = Vec::new();

        write_size_header!($desc, error);
        error.extend_from_slice($desc);

        if $details.len() > 0 {
            write_size!($details.len() + offset_by_size($details.len()), error);
        }

        write_size_header!($details, error);
        error.extend_from_slice($details);

        error.extend_from_slice($code);

        Error::ProgramError(error)
    }}
}

macro_rules! error_database {
    ($err: expr) => {{
        let vec = Vec::new();

        error_program!(
            $err.description().as_bytes(),
            &vec,
            ERROR_DATABASE
        )
    }}
}

macro_rules! error_no_transaction {
    () => {{
        let vec = Vec::new();
        error_program!(
            "No transaction".as_bytes(),
            &vec,
            ERROR_NO_TX
        )
    }}
}

macro_rules! error_unknown_key {
    ($key: expr) => {{
        error_program!(
            "Unknown key".as_bytes(),
            $key,
            ERROR_UNKNOWN_KEY
        )
    }}
}

macro_rules! error_duplicate_key {
    ($key: expr) => {{
        error_program!(
            "Duplicate key".as_bytes(),
            $key,
            ERROR_DUPLICATE_KEY
        )
    }}
}

macro_rules! error_decoding {
    () => {{
        let vec = Vec::new();
        error_program!(
            "Decoding error".as_bytes(),
            &vec,
            ERROR_DECODING
        )
    }}
}

macro_rules! error_empty_stack {
    () => {{
        let vec = Vec::new();
        error_program!(
            "Empty stack".as_bytes(),
            &vec,
            ERROR_EMPTY_STACK
        )
    }}
}

macro_rules! error_invalid_value {
    ($value: expr) => {{
        error_program!(
            "Invalid value".as_bytes(),
            &$value,
            ERROR_INVALID_VALUE
        )
    }}
}

macro_rules! error_unknown_word {
    ($word: expr) => { {
        let (_, w) = binparser::word_or_internal_word($word).unwrap();

        let word = match str::from_utf8(&w[1..]) {
            Ok(word) => word,
            Err(_) => "Error parsing word"
        };

        let desc = format!("Unknown word: {}", word);
        let desc_bytes = desc.as_bytes();

        error_program!(
            desc_bytes,
            $word,
            ERROR_UNKNOWN_WORD
        )
    } }
}

macro_rules! write_size_header {
    ($bytes: expr, $vec: expr) => {{
        write_size!($bytes.len(), $vec);
    }};
}

macro_rules! write_size {
    ($size: expr, $vec: expr) => {{
        let mut header = vec![0;offset_by_size($size)];
        write_size_into_slice!($size, header.as_mut_slice());
        $vec.append(&mut header);
    }};
}

macro_rules! alloc_slice {
    ($size: expr, $env: expr) => {{
        let slice = $env.alloc($size);
        if slice.is_err() {
            return Err(slice.unwrap_err());
        }
        slice.unwrap()
    }};
}

macro_rules! alloc_and_write {
    ($bytes: expr, $env: expr) => {{
        let slice = alloc_slice!($bytes.len(), $env);
        slice.copy_from_slice($bytes);
        slice
    }};
}

#[cfg(test)]
macro_rules! eval {
        ($script: expr, $env: ident, $expr: expr) => {
           eval!($script, $env, _result, $expr);
        };
        ($script: expr, $env: ident, $result: ident, $expr: expr) => {
           eval!($script, $env, $result, publisher_accessor, {}, $expr);
        };
        ($script: expr, $env: ident, $result: ident, $publisher_accessor: ident, {$($init: tt)*}, $expr: expr) => {
          {
            let dir = TempDir::new("pumpkindb").unwrap();
            let path = dir.path().to_str().unwrap();
            fs::create_dir_all(path).expect("can't create directory");
            let env = unsafe {
                lmdb::EnvBuilder::new()
                    .expect("can't create env builder")
                    .open(path, lmdb::open::NOTLS, 0o600)
                    .expect("can't open env")
            };

            let db = storage::Storage::new(&env);
            crossbeam::scope(|scope| {
                let mut publisher = pubsub::Publisher::new();
                let $publisher_accessor = publisher.accessor();
                let publisher_thread = scope.spawn(move || publisher.run());
                $($init)*
                let publisher_clone = $publisher_accessor.clone();
                let (sender_sender, receiver) = mpsc::sync_channel(0);
                let handle = scope.spawn(move || {
                    let mut scheduler = Scheduler::new(
                        &db,
                        publisher_clone,
                        sender_sender,
                        );
                    scheduler.run()
                });
                let sender = receiver.recv().unwrap();
                let script = parse($script).unwrap();
                let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(),
                                    script.clone(), callback));
                match receiver.recv() {
                   Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                      let _ = sender.send(RequestMessage::Shutdown);
                      $publisher_accessor.shutdown();
                      let $result = Ok::<(), Error>(());
                      let mut stack_ = Vec::with_capacity(stack.len());
                      for i in 0..(&stack).len() {
                          stack_.push((&stack[i]).as_slice());
                      }
                      let mut $env = Env::new_with_stack(stack_, stack_size).unwrap();
                      $expr;
                   }
                   Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                      let _ = sender.send(RequestMessage::Shutdown);
                      $publisher_accessor.shutdown();
                      let $result = Err::<(), Error>(err);
                      let stack = stack.unwrap();
                      let mut stack_ = Vec::with_capacity(stack.len());
                      for i in 0..(&stack).len() {
                          stack_.push((&stack)[i].as_slice());
                      }
                      let mut $env = Env::new_with_stack(stack_, stack_size.unwrap()).unwrap();
                      $expr;
                   }
                   Err(err) => {
                      let _ = sender.send(RequestMessage::Shutdown);
                      $publisher_accessor.shutdown();
                      panic!("recv error: {:?}", err);
                   }
                }
                let _ = handle.join();
                let _ = publisher_thread.join();
          });
        };
      }
}

#[cfg(test)]
macro_rules! bench_eval {
        ($script: expr, $b: expr) => {
          {
            let dir = TempDir::new("pumpkindb").unwrap();
            let path = dir.path().to_str().unwrap();
            fs::create_dir_all(path).expect("can't create directory");
            let env = unsafe {
                let mut builder = lmdb::EnvBuilder::new().expect("can't create env builder");
                builder.set_mapsize(1024 * 1024 * 1024).expect("can't set mapsize");
                builder.open(path, lmdb::open::NOTLS, 0o600).expect("can't open env")
            };

            let db = storage::Storage::new(&env);
            crossbeam::scope(|scope| {
                let mut publisher = pubsub::Publisher::new();
                let publisher_accessor = publisher.accessor();
                let publisher_accessor_ = publisher.accessor();
                let publisher_thread = scope.spawn(move || publisher.run());
                let publisher_clone = publisher_accessor.clone();
                let (sender_sender, receiver) = mpsc::sync_channel(0);
                let handle = scope.spawn(move || {
                    let mut scheduler = Scheduler::new(
                        &db,
                        publisher_clone,
                        sender_sender,
                    );
                    scheduler.run()
                });
                let sender = receiver.recv().unwrap();
                let sender_ = sender.clone();
                let script = parse($script).unwrap();
                $b.iter(move || {
                    let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                    let _ = sender.send(RequestMessage::ScheduleEnv(EnvId::new(),
                                        script.clone(), callback));
                    match receiver.recv() {
                       Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => (),
                       Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                          let _ = sender.send(RequestMessage::Shutdown);
                          publisher_accessor.shutdown();
                          panic!("error: {:?}", err);
                       }
                       Err(err) => {
                          let _ = sender.send(RequestMessage::Shutdown);
                          publisher_accessor.shutdown();
                          panic!("recv error: {:?}", err);
                       }
                    }
                });
                let _ = sender_.send(RequestMessage::Shutdown);
                publisher_accessor_.shutdown();
                let _ = handle.join();
                let _ = publisher_thread.join();
          });
        };
      }
}

#[cfg(test)]
macro_rules! data {
    ($ptr:expr) => {
        {
          let (_, size) = binparser::data_size($ptr).unwrap();
          &$ptr[offset_by_size(size)..$ptr.len()]
        }
    };
}

#[cfg(test)]
macro_rules! parsed_data {
        ($s: expr) => {
           data!(parse($s).unwrap().as_slice())
        };
}

#[cfg(test)]
macro_rules! assert_error {
    ($result: expr, $expected: expr) => {{
        assert!($result.is_err());
        let error = $result.err().unwrap();
        assert!(matches!(error, Error::ProgramError(_)));
        if let Error::ProgramError(inner) = error {
            assert_eq!(inner, parsed_data!($expected));
        } else {
        }
    }};
}