use super::CasmProgram;
use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            stack::{Offset, StackError, STACK_SIZE},
            Memory, MemoryAddress,
        },
        casm::operation::OpPrimitive,
        scheduler::Thread,
        vm::{Executable, RuntimeError},
    },
};
use num_traits::ToBytes;
use std::{cell::Cell, mem};

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl Executable for Locate {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match &self.address {
            MemoryAddress::Heap { offset } => {
                // let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                todo!();
                thread.env.program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset, level } => {
                if let Offset::FE(stack_idx, heap_idx) = offset {
                    let heap_address = thread
                        .env
                        .stack
                        .read(Offset::FP(*stack_idx), *level, 8)
                        .map_err(|err| err.into())?;
                    let data = TryInto::<&[u8; 8]>::try_into(heap_address.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                    let heap_address = u64::from_le_bytes(*data);
                    let data = (heap_address + *heap_idx as u64).to_le_bytes();
                    let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                    thread.env.program.incr();
                    return Ok(());
                }
                let offset = thread
                    .env
                    .stack
                    .compute_absolute_address(*offset, *level)
                    .map_err(|e| e.into())?;
                let data = (offset as u64).to_le_bytes();
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LocateNextUTF8Char {
    RuntimeNext,
    RuntimeAtIdx,
}

impl Executable for LocateNextUTF8Char {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        thread.env.program.incr();
        match self {
            LocateNextUTF8Char::RuntimeNext => {
                let pointer = OpPrimitive::get_num8::<u64>(&thread.memory())?;

                let address = {
                    if pointer < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(pointer as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: pointer as usize,
                        }
                    }
                };
                let offset = match address {
                    MemoryAddress::Heap { offset } => {
                        let (_, offset) = thread
                            .runtime
                            .heap
                            .read_utf8(offset, 1)
                            .map_err(|err| err.into())?;
                        offset
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let (_, offset) = thread
                            .env
                            .stack
                            .read_utf8(offset, level, 1)
                            .map_err(|err| err.into())?;
                        offset
                    }
                };

                let _ = thread
                    .env
                    .stack
                    .push_with(&(pointer + offset as u64).to_le_bytes());

                Ok(())
            }
            LocateNextUTF8Char::RuntimeAtIdx => {
                /* */
                let idx = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let pointer = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let address = {
                    if pointer < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(pointer as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: pointer as usize,
                        }
                    }
                };

                let size = match address {
                    MemoryAddress::Heap { offset } => {
                        let (_, size) = thread
                            .runtime
                            .heap
                            .read_utf8(offset, idx as usize)
                            .map_err(|err| err.into())?;
                        size
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let (_, size) = thread
                            .env
                            .stack
                            .read_utf8(offset, level, idx as usize)
                            .map_err(|err| err.into())?;
                        size
                    }
                };

                let _ = thread
                    .env
                    .stack
                    .push_with(&(pointer + size as u64).to_le_bytes());

                Ok(())
            }
        }
    }
}
