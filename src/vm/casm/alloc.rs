use num_traits::ToBytes;

use super::CasmProgram;
use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            stack::{Offset, StackSlice},
            Memory, MemoryAddress,
        },
        casm::operation::OpPrimitive,
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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            Alloc::Heap { size } => {
                let address = memory.heap.alloc(*size).map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                let data = (address as u64).to_le_bytes().to_vec();
                let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = memory.stack.push(*size).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StackFrame {
    Clean,
    Set {
        return_size: usize,
        params_size: usize,
        cursor_offset: usize,
    },
    Return {
        return_size: usize,
    },
}

impl Executable for StackFrame {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            StackFrame::Clean => {
                program.cursor.set(memory.stack.registers.link.get());
                let _ = memory.stack.clean().map_err(|e| e.into())?;
                Ok(())
            }
            StackFrame::Set {
                return_size,
                params_size,
                cursor_offset,
            } => {
                let _ = memory
                    .stack
                    .frame(
                        *return_size,
                        *params_size,
                        program.cursor.get() + cursor_offset,
                    )
                    .map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            StackFrame::Return { return_size } => {
                let return_data = memory.stack.pop(*return_size).map_err(|e| e.into())?;

                let _ = memory
                    .stack
                    .write(Offset::FB(0), AccessLevel::Direct, &return_data)
                    .map_err(|e| e.into())?;
                program.cursor.set(memory.stack.registers.link.get());
                let _ = memory.stack.clean().map_err(|e| e.into())?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: usize },
}

impl Executable for Access {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            Access::Static { address, size } => match address {
                MemoryAddress::Heap => {
                    let address = OpPrimitive::get_num8::<u64>(memory)? as usize;
                    let _data = memory.heap.read(address, *size).map_err(|err| err.into())?;
                    todo!("Copy data onto stack");
                    program.incr();
                    Ok(())
                }
                MemoryAddress::Stack { offset, level } => {
                    let data = memory
                        .stack
                        .read(*offset, *level, *size)
                        .map_err(|err| err.into())?;
                    // Copy data onto stack;
                    let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                    program.incr();
                    Ok(())
                }
            },
            Access::Runtime { size: _ } => {
                let _address = OpPrimitive::get_num8::<u64>(memory)?;
                let _address = {
                    todo!("Convert u64 to memory address by differenting stack pointer and heap pointer");
                };
                program.incr();
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
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let data = memory
            .stack
            .read(
                self.stack_slice.offset,
                AccessLevel::Direct,
                self.stack_slice.size,
            )
            .map_err(|err| err.into())?;

        match self.address {
            MemoryAddress::Heap => {
                let address = OpPrimitive::get_num8::<u64>(memory)? as usize;
                let _ = memory.heap.write(address, &data).map_err(|e| e.into())?;
            }
            MemoryAddress::Stack { offset, level } => {
                let _ = memory
                    .stack
                    .write(offset, level, &data)
                    .map_err(|e| e.into())?;
            }
        };

        program.incr();
        Ok(())
    }
}
