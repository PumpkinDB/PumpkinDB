// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_export]
macro_rules! builtins {
    ($file: expr) => {
    lazy_static! {
      static ref BUILTIN_FILE: &'static [u8] = include_bytes!($file);

      static ref BUILTIN_DEFS: Vec<Vec<u8>> = ::pumpkinscript::textparser::programs(*BUILTIN_FILE).unwrap().1;

      static ref BUILTINS: ::std::collections::BTreeMap<&'static [u8], &'static [u8]> = {
          let mut map = ::std::collections::BTreeMap::new();
          let ref defs : Vec<Vec<u8>> = *BUILTIN_DEFS;
          for definition in defs {
              match ::pumpkinscript::binparser::instruction(&definition) {
                  ::pumpkinscript::ParseResult::Done(&[0x81, b':', ref rest..], _) => {
                      let instruction = &definition[0..definition.len() - rest.len() - 2];
                      map.insert(instruction, rest);
                  },
                  other => panic!("builtin definition parse error {:?}", other)
              }
          }
          map
      };
    }};
}

#[macro_export]
macro_rules! handle_builtins {
    () => {
        #[inline]
        fn handle_builtins(&mut self,
                           env: &mut Env<'a>,
                           instruction: &'a [u8],
                           _: EnvId)
                           -> PassResult<'a> {
            match BUILTINS.get(instruction) {
               Some(val) => {
                  env.program.push(val);
                  Ok(())
               },
               None => Err(Error::UnknownInstruction),
            }
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! return_unless_instructions_equal {
    ($instruction: expr, $exp: expr) => {
        if $instruction != $exp {
            return Err(Error::UnknownInstruction)
        }
    };
}

#[macro_export]
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

#[macro_export]
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

#[macro_export]
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

#[macro_export]
macro_rules! error_unknown_key {
    ($key: expr) => {{
        error_program!(
            "Unknown key".as_bytes(),
            $key,
            ERROR_UNKNOWN_KEY
        )
    }}
}

#[macro_export]
macro_rules! error_duplicate_key {
    ($key: expr) => {{
        error_program!(
            "Duplicate key".as_bytes(),
            $key,
            ERROR_DUPLICATE_KEY
        )
    }}
}

#[macro_export]
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

#[macro_export]
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

#[macro_export]
macro_rules! error_invalid_value {
    ($value: expr) => {{
        error_program!(
            "Invalid value".as_bytes(),
            &$value,
            ERROR_INVALID_VALUE
        )
    }}
}

#[macro_export]
macro_rules! error_no_value {
    () => {{
        let vec = Vec::new();
        error_program!(
            "No value".as_bytes(),
            &vec,
            ERROR_NO_VALUE
        )
    }}
}

#[macro_export]
macro_rules! error_unknown_instruction {
    ($instruction: expr) => { {
        let (_, w) = binparser::instruction_or_internal_instruction($instruction).unwrap();

        let instruction = match str::from_utf8(&w[1..]) {
            Ok(instruction) => instruction,
            Err(_) => "Error parsing instruction"
        };

        let desc = format!("Unknown instruction: {}", instruction);
        let desc_bytes = desc.as_bytes();

        error_program!(
            desc_bytes,
            $instruction,
            ERROR_UNKNOWN_INSTRUCTION
        )
    } }
}

#[macro_export]
macro_rules! alloc_slice {
    ($size: expr, $env: expr) => {{
        let slice = $env.alloc($size);
        if slice.is_err() {
            return Err(slice.unwrap_err());
        }
        slice.unwrap()
    }};
}

#[macro_export]
macro_rules! alloc_and_write {
    ($bytes: expr, $env: expr) => {{
        let slice = alloc_slice!($bytes.len(), $env);
        slice.copy_from_slice($bytes);
        slice
    }};
}

// TODO: use or remove?
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! eval {
        ($script: expr, $env: ident, $expr: expr) => {
           eval!($script, $env, _result, $expr);
        };
        ($script: expr, $env: ident, $result: ident, $expr: expr) => {{
           let (sender, receiver) = mpsc::channel();
           eval!($script, $env, $result, sender, receiver, $expr);
        }};
        ($script: expr, $env: ident, $result: ident, $sender: expr, $receiver: ident, $expr: expr) => {
          {
            use $crate::script::SchedulerHandle;
            let dir = TempDir::new("pumpkindb").unwrap();
            let path = dir.path().to_str().unwrap();
            fs::create_dir_all(path).expect("can't create directory");
            let env = unsafe {
                lmdb::EnvBuilder::new()
                    .expect("can't create env builder")
                    .open(path, lmdb::open::NOTLS, 0o600)
                    .expect("can't open env")
            };

            let db = Arc::new(storage::Storage::new(&env));
            crossbeam::scope(|scope| {
                let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
                let region = nvmem.claim(20).unwrap();
                let timestamp = Arc::new(timestamp::Timestamp::new(region));
                let mut simple = messaging::Simple::new();
                let messaging_accessor = simple.accessor();
                let publisher_thread = scope.spawn(move || simple.run());
                let publisher_clone = messaging_accessor.clone();
                let subscriber_clone = messaging_accessor.clone();
                let timestamp_clone = timestamp.clone();
                let (mut scheduler, sender) = Scheduler::new(
                    dispatcher::StandardDispatcher::new(db.clone(), publisher_clone.clone(), subscriber_clone.clone(),
                    timestamp_clone));
                let handle = scope.spawn(move || scheduler.run());
                let script = parse($script).unwrap();
                let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                sender.schedule_env(EnvId::new(),
                                    script.clone(), callback, Box::new($sender));
                match receiver.recv() {
                   Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => {
                      sender.shutdown();
                      messaging_accessor.shutdown();
                      let $result = Ok::<(), Error>(());
                      let mut stack_ = Vec::with_capacity(stack.len());
                      for i in 0..(&stack).len() {
                          stack_.push((&stack[i]).as_slice());
                      }
                      let mut $env = Env::new_with_stack(stack_).unwrap();
                      $expr;
                   }
                   Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                      sender.shutdown();
                      messaging_accessor.shutdown();
                      let $result = Err::<(), Error>(err);
                      let stack = stack.unwrap();
                      let mut stack_ = Vec::with_capacity(stack.len());
                      for i in 0..(&stack).len() {
                          stack_.push((&stack)[i].as_slice());
                      }
                      let mut $env = Env::new_with_stack(stack_).unwrap();
                      $expr;
                   }
                   Err(err) => {
                      sender.shutdown();
                      messaging_accessor.shutdown();
                      panic!("recv error: {:?}", err);
                   }
                }
                let _ = handle.join();
                let _ = publisher_thread.join();
          });
        };
      }
}

// TODO: use or remove?
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! bench_eval {
        ($script: expr, $b: expr) => {
          {
            use $crate::script::SchedulerHandle;
            let dir = TempDir::new("pumpkindb").unwrap();
            let path = dir.path().to_str().unwrap();
            fs::create_dir_all(path).expect("can't create directory");
            let env = unsafe {
                let mut builder = lmdb::EnvBuilder::new().expect("can't create env builder");
                builder.set_mapsize(1024 * 1024 * 1024).expect("can't set mapsize");
                builder.open(path, lmdb::open::NOTLS, 0o600).expect("can't open env")
            };

            let db = Arc::new(storage::Storage::new(&env));
            let cpus = ::num_cpus::get();
            crossbeam::scope(|scope| {
                let mut simple = messaging::Simple::new();
                let messaging_accessor = simple.accessor();
                let messaging_accessor_ = simple.accessor();
                let simple_thread = scope.spawn(move || simple.run());
                let mut nvmem = MmapedFile::new_anonymous(20).unwrap();
                let region = nvmem.claim(20).unwrap();
                let timestamp = Arc::new(timestamp::Timestamp::new(region));

                let mut handles = vec![];
                let mut senders = vec![];
                for i in 0..cpus {
                    let publisher_clone = messaging_accessor.clone();
                    let subscriber_clone = messaging_accessor.clone();
                    let timestamp_clone = timestamp.clone();
                    let (mut scheduler, sender) = Scheduler::new(
                        dispatcher::StandardDispatcher::new(db.clone(), publisher_clone.clone(), subscriber_clone.clone(),
                        timestamp_clone));
                    let storage = db.clone();
                    let handle = scope.spawn(move || scheduler.run());
                    handles.push(handle);
                    senders.push(sender.clone());
                }
                let original_senders = senders.clone();
                let script = parse($script).unwrap();
                $b.iter(move || {
                    let (callback, receiver) = mpsc::channel::<ResponseMessage>();
                    let (sender0, _) = mpsc::channel();
                    let _ = senders.clone().schedule_env(EnvId::new(),
                                           script.clone(), callback, Box::new(sender0));
                    match receiver.recv() {
                       Ok(ResponseMessage::EnvTerminated(_, stack, stack_size)) => (),
                       Ok(ResponseMessage::EnvFailed(_, err, stack, stack_size)) => {
                          senders.shutdown();
                          messaging_accessor.shutdown();
                          panic!("error: {:?}", err);
                       }
                       Err(err) => {
                          senders.shutdown();
                          messaging_accessor.shutdown();
                          panic!("recv error: {:?}", err);
                       }
                    }
                });
                original_senders.shutdown();
                messaging_accessor_.shutdown();
                for handle in handles {
                   handle.join();
                }
                simple_thread.join();
          });
        };
      }
}

// TODO: use or remove?
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! data {
    ($ptr:expr) => {
        {
          let (_, size) = binparser::data_size($ptr).unwrap();
          &$ptr[offset_by_size(size)..$ptr.len()]
        }
    };
}

// TODO: use or remove?
#[allow(unused_macros)]
#[cfg(test)]
macro_rules! parsed_data {
        ($s: expr) => {
           data!(parse($s).unwrap().as_slice())
        };
}

// TODO: use or remove?
#[allow(unused_macros)]
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
