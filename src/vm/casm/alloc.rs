use num_traits::ToBytes;

use super::CasmProgram;
use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            stack::{Offset, StackSlice, STACK_SIZE},
            Memory, MemoryAddress,
        },
        casm::operation::OpPrimitive,
        scheduler::Thread,
        vm::{Executable, RuntimeError},
    },
};
use std::{cell::Cell, mem};
#[derive(Debug, Clone)]
pub enum Alloc {
    Heap { size: usize },
    Stack { size: usize },
}

impl Executable for Alloc {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            Alloc::Heap { size } => {
                let address = thread.runtime.heap.alloc(*size).map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                let data = (address as u64).to_le_bytes().to_vec();
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = thread.env.stack.push(*size).map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StackFrame {
    Clean,
    SoftClean,
    Break,
    Continue,
    Set {
        params_size: usize,
        cursor_offset: usize,
    },
    Return {
        return_size: Option<usize>,
    },
}

impl Executable for StackFrame {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            StackFrame::Break => {
                // let idx = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                // thread.env.program.cursor_set(idx);

                thread
                    .env
                    .program
                    .cursor
                    .set(thread.env.stack.registers.link.get());

                let _ = thread.env.stack.clean().map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&0u64.to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&[2u8]).map_err(|e| e.into())?; /* return flag set to BREAK */

                Ok(())
            }
            StackFrame::Continue => {
                // let idx = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                // thread.env.program.cursor_set(idx);

                thread
                    .env
                    .program
                    .cursor
                    .set(thread.env.stack.registers.link.get());

                let _ = thread.env.stack.clean().map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&0u64.to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&[3u8]).map_err(|e| e.into())?; /* return flag set to CONTINUE */

                Ok(())
            }
            StackFrame::SoftClean => {
                thread.env.program.incr();
                let _ = thread.env.stack.clean().map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&0u64.to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&[0u8]).map_err(|e| e.into())?; /* return flag set to false */
                Ok(())
            }
            StackFrame::Clean => {
                thread
                    .env
                    .program
                    .cursor
                    .set(thread.env.stack.registers.link.get());
                let _ = thread.env.stack.clean().map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&0u64.to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&[0u8]).map_err(|e| e.into())?; /* return flag set to false */
                Ok(())
            }
            StackFrame::Set {
                params_size,
                cursor_offset,
            } => {
                let _ = thread
                    .env
                    .stack
                    .frame(
                        *params_size,
                        thread.env.program.cursor.get() + cursor_offset,
                    )
                    .map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
            StackFrame::Return { return_size } => {
                let return_size = match return_size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                        size as usize
                    }
                };
                let return_data = thread.env.stack.pop(return_size).map_err(|e| e.into())?;

                thread
                    .env
                    .program
                    .cursor
                    .set(thread.env.stack.registers.link.get());
                let _ = thread.env.stack.clean().map_err(|e| e.into())?;

                let _ = thread
                    .env
                    .stack
                    .push_with(&return_data)
                    .map_err(|e| e.into())?;
                let _ = thread
                    .env
                    .stack
                    .push_with(&return_size.to_le_bytes())
                    .map_err(|e| e.into())?;
                let _ = thread.env.stack.push_with(&[1u8]).map_err(|e| e.into())?; /* return flag set to true */

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: Option<usize> },
}

impl Executable for Access {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let (address, size) = match self {
            Access::Static { address, size } => (*address, *size),
            Access::Runtime { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                        size as usize
                    }
                };
                let address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                let address = {
                    if address < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(address as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: address as usize,
                        }
                    }
                };
                (address, size)
            }
        };
        match address {
            MemoryAddress::Heap { offset } => {
                // let address = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                let data = thread
                    .runtime
                    .heap
                    .read(offset, size)
                    .map_err(|err| err.into())?;
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset, level } => {
                if let Offset::FE(stack_idx, heap_udx) = offset {
                    let heap_address = thread
                        .env
                        .stack
                        .read(Offset::FP(stack_idx), level, 8)
                        .map_err(|err| err.into())?;
                    let data = TryInto::<&[u8; 8]>::try_into(heap_address.as_slice())
                        .map_err(|_| RuntimeError::Deserialization)?;
                    let heap_address = u64::from_le_bytes(*data);

                    let data = thread
                        .runtime
                        .heap
                        .read(heap_address as usize + heap_udx, size)
                        .map_err(|err| err.into())?;
                    let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                    thread.env.program.incr();
                    return Ok(());
                }

                let data = thread
                    .env
                    .stack
                    .read(offset, level, size)
                    .map_err(|err| err.into())?;
                // dbg!(&data);
                // Copy data onto stack;
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                thread.env.program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    address: MemoryAddress,
    stack_slice: StackSlice,
}

impl Executable for Assign {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let data = thread
            .env
            .stack
            .read(
                self.stack_slice.offset,
                AccessLevel::Direct,
                self.stack_slice.size,
            )
            .map_err(|err| err.into())?;

        match self.address {
            MemoryAddress::Heap { offset } => {
                // let address = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                let _ = thread
                    .runtime
                    .heap
                    .write(offset, &data)
                    .map_err(|e| e.into())?;
            }
            MemoryAddress::Stack { offset, level } => {
                let _ = thread
                    .env
                    .stack
                    .write(offset, level, &data)
                    .map_err(|e| e.into())?;
            }
        };

        thread.env.program.incr();
        Ok(())
    }
}
