// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pumpkinscript::{parse_bin, binparser};

use super::{Env, EnvId, Dispatcher, PassResult, Error, ERROR_EMPTY_STACK, ERROR_INVALID_VALUE,
            offset_by_size, STACK_TRUE, STACK_FALSE, TryInstruction};

use super::mod_stack::{PUSH, POP};
use super::mod_queue::{TO_BQ, FROM_BQ};
use std::marker::PhantomData;

use pumpkinscript;
use num_bigint::BigUint;
use num_traits::Zero;

// Category: Control flow
#[cfg(feature = "scoped_dictionary")]
instruction!(EVAL_SCOPED, b"\x8BEVAL/SCOPED");
#[cfg(feature = "scoped_dictionary")]
instruction!(SCOPE_END, b"\x80\x8BEVAL/SCOPED"); // internal instruction
instruction!(DOWHILE, b"\x87DOWHILE");
instruction!(TIMES, b"\x85TIMES");
instruction!(EVAL, b"\x84EVAL");
instruction!(EVAL_VALIDP, b"\x8BEVAL/VALID?");
instruction!(SET, b"\x83SET");
instruction!(DEF, b"\x83DEF");
instruction!(IF, b"\x82IF"); // for reference, implemented in builtins
instruction!(IFELSE, b"\x86IFELSE");

// Category: Logical operations
instruction!(NOT, (a => c), b"\x83NOT");
instruction!(AND, (a, b => c), b"\x83AND");
instruction!(OR, (a, b => c), b"\x82OR");

// Category: experimental features
instruction!(FEATUREQ, (a => b), b"\x88FEATURE?");

builtins!("mod_core.psc");

pub struct Handler<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Dispatcher<'a> for Handler<'a> {
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.handle_builtins(env, instruction, pid)
        .if_unhandled_try(|| self.handle_dowhile(env, instruction, pid))
        .if_unhandled_try(|| self.handle_times(env, instruction, pid))
        .if_unhandled_try(|| self.handle_scope_end(env, instruction, pid))
        .if_unhandled_try(|| self.handle_eval(env, instruction, pid))
        .if_unhandled_try(|| self.handle_eval_validp(env, instruction, pid))
        .if_unhandled_try(|| self.handle_eval_scoped(env, instruction, pid))
        .if_unhandled_try(|| self.handle_set(env, instruction, pid))
        .if_unhandled_try(|| self.handle_def(env, instruction, pid))
        .if_unhandled_try(|| self.handle_not(env, instruction, pid))
        .if_unhandled_try(|| self.handle_and(env, instruction, pid))
        .if_unhandled_try(|| self.handle_or(env, instruction, pid))
        .if_unhandled_try(|| self.handle_ifelse(env, instruction, pid))
        .if_unhandled_try(|| self.handle_featurep(env, instruction, pid))
        .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

impl<'a> Handler<'a> {
    pub fn new() -> Self {
        Handler {
            phantom: PhantomData,
        }
    }

    handle_builtins!();

    #[inline]
    fn handle_not(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, NOT);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;

        if a == STACK_TRUE {
            env.push(STACK_FALSE);
        } else if a == STACK_FALSE {
            env.push(STACK_TRUE);
        } else {
            return Err(error_invalid_value!(a));
        }

        Ok(())
    }

    #[inline]
    fn handle_and(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, AND);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err(error_invalid_value!(a));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err(error_invalid_value!(b));
        }

        if a == STACK_TRUE && b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else if a == STACK_FALSE || b == STACK_FALSE {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_or(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, OR);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        let b = env.pop().ok_or_else(|| error_empty_stack!())?;

        if !(a == STACK_TRUE || a == STACK_FALSE) {
            return Err(error_invalid_value!(a));
        }
        if !(b == STACK_TRUE || b == STACK_FALSE) {
            return Err(error_invalid_value!(b));
        }

        if a == STACK_TRUE || b == STACK_TRUE {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }

        Ok(())
    }

    #[inline]
    fn handle_ifelse(&mut self,
                     env: &mut Env<'a>,
                     instruction: &'a [u8],
                     _: EnvId)
                     -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, IFELSE);
        let else_ = env.pop().ok_or_else(|| error_empty_stack!())?;
        let then = env.pop().ok_or_else(|| error_empty_stack!())?;
        let cond = env.pop().ok_or_else(|| error_empty_stack!())?;

        if cond == STACK_TRUE {
            env.program.push(then);
            Ok(())
        } else if cond == STACK_FALSE {
            env.program.push(else_);
            Ok(())
        } else {
            Err(error_invalid_value!(cond))
        }
    }

    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_eval_scoped(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, EVAL_SCOPED);
        env.push_dictionary();
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        env.program.push(SCOPE_END);
        env.program.push(a);
        Ok(())
    }

    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_eval_scoped(&mut self, _: &Env<'a>, _: &'a [u8], _: EnvId) -> PassResult<'a> {
        Err(Error::UnknownInstruction)
    }


    #[inline]
    #[cfg(feature = "scoped_dictionary")]
    fn handle_scope_end(&mut self,
                        env: &mut Env<'a>,
                        instruction: &'a [u8],
                        _: EnvId)
                        -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, SCOPE_END);
        env.pop_dictionary();
        Ok(())
    }


    #[inline]
    #[cfg(not(feature = "scoped_dictionary"))]
    fn handle_scope_end(&mut self, _: &mut Env<'a>, _: &'a [u8], _: EnvId) -> PassResult<'a> {
        Err(Error::UnknownInstruction)
    }

    #[inline]
    fn handle_eval(&mut self,
                   env: &mut Env<'a>,
                   instruction: &'a [u8],
                   _: EnvId)
                   -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, EVAL);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        env.program.push(a);
        Ok(())
    }

    #[inline]
    fn handle_eval_validp(&mut self,
                          env: &mut Env<'a>,
                          instruction: &'a [u8],
                          _: EnvId)
                          -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, EVAL_VALIDP);
        let a = env.pop().ok_or_else(|| error_empty_stack!())?;
        if parse_bin(a).is_ok() {
            env.push(STACK_TRUE);
        } else {
            env.push(STACK_FALSE);
        }
        Ok(())
    }

    #[inline]
    fn handle_dowhile(&mut self,
                      env: &mut Env<'a>,
                      instruction: &'a [u8],
                      _: EnvId)
                      -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, DOWHILE);
        let v = env.pop().ok_or_else(|| error_empty_stack!())?;
        let mut v1 = vec![];
        v1.extend_from_slice(PUSH);
        v1.extend_from_slice(v);
        v1.extend_from_slice(TO_BQ);
        v1.extend_from_slice(POP);
        v1.extend_from_slice(FROM_BQ);


        let mut vec = Vec::new();

        let mut header = vec![0;offset_by_size(v.len() + DOWHILE.len() + offset_by_size(v.len()))];
        write_size_into_slice!(offset_by_size(v.len()) + v.len() + DOWHILE.len(),
                               header.as_mut_slice());
        vec.append(&mut header);

        // inject code closure size
        let mut header = vec![0;offset_by_size(v.len())];
        write_size_into_slice!(v.len(), header.as_mut_slice());
        vec.append(&mut header);

        // inject code closure
        vec.extend_from_slice(v);
        // inject DOWHILE
        vec.extend_from_slice(DOWHILE);
        // inject IF
        vec.extend_from_slice(IF);

        let slice = alloc_and_write!(vec.as_slice(), env);
        env.program.push(slice);
        let code = alloc_and_write!(&v1, env);
        env.program.push(code);

        Ok(())
    }

    #[inline]
    fn handle_times(&mut self,
                    env: &mut Env<'a>,
                    instruction: &'a [u8],
                    _: EnvId)
                    -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, TIMES);
        let count = env.pop().ok_or_else(|| error_empty_stack!())?;

        let v = env.pop().ok_or_else(|| error_empty_stack!())?;

        let counter = BigUint::from_bytes_be(count);
        use num_iter;
        for _ in num_iter::range(BigUint::zero(), counter) {
            env.program.push(POP);
            env.program.push(v);
            env.program.push(PUSH);
        }
        Ok(())
    }

    #[inline]
    fn handle_set(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, SET);
        let instruction = env.pop().ok_or_else(|| error_empty_stack!())?;
        let value = env.pop().ok_or_else(|| error_empty_stack!())?;
        match binparser::instruction(instruction) {
            pumpkinscript::ParseResult::Done(_, _) => {
                let slice = alloc_slice!(value.len() + offset_by_size(value.len()), env);
                write_size_into_slice!(value.len(), slice);
                let offset = offset_by_size(value.len());
                slice[offset..offset + value.len()].copy_from_slice(value);
                #[cfg(feature = "scoped_dictionary")]
                {
                    let mut dict = env.dictionary.pop().unwrap();
                    dict.insert(instruction, slice);
                    env.dictionary.push(dict);
                }
                #[cfg(not(feature = "scoped_dictionary"))]
                env.dictionary.insert(instruction, slice);
                Ok(())
            }
            _ => Err(error_invalid_value!(instruction)),
        }
    }

    #[inline]
    fn handle_def(&mut self, env: &mut Env<'a>, instruction: &'a [u8], _: EnvId) -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, DEF);
        let instruction = env.pop().ok_or_else(|| error_empty_stack!())?;
        let value = env.pop().ok_or_else(|| error_empty_stack!())?;
        match binparser::instruction(instruction) {
            pumpkinscript::ParseResult::Done(_, _) => {
                #[cfg(feature = "scoped_dictionary")]
                {
                    let mut dict = env.dictionary.pop().unwrap();
                    dict.insert(instruction, value);
                    env.dictionary.push(dict);
                }
                #[cfg(not(feature = "scoped_dictionary"))]
                env.dictionary.insert(instruction, value);
                Ok(())
            }
            _ => Err(error_invalid_value!(instruction)),
        }
    }

    #[inline]
    #[allow(unused_variables)]
    fn handle_featurep(&mut self,
                       env: &mut Env<'a>,
                       instruction: &'a [u8],
                       _: EnvId)
                       -> PassResult<'a> {
        return_unless_instructions_equal!(instruction, FEATUREQ);
        let name = env.pop().ok_or_else(|| error_empty_stack!())?;

        #[cfg(feature = "scoped_dictionary")]
        {
            if name == "scoped_dictionary".as_bytes() {
                env.push(STACK_TRUE);
                return Ok(());
            }
        }

        env.push(STACK_FALSE);

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_must_use, unused_mut)]
mod tests {

    use pumpkinscript::parse;
    use messaging;
    use nvmem::{MmapedFile};
    use script::{Scheduler, ResponseMessage, EnvId, dispatcher};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::fs;
    use tempdir::TempDir;
    use lmdb;
    use crossbeam;
    use storage;
    use timestamp;

    const _EMPTY: &'static [u8] = b"";

    use test::Bencher;

    #[bench]
    fn times(b: &mut Bencher) {
        bench_eval!("[1 DROP] 1000 TIMES", b);
    }

}
