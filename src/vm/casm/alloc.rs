use num_traits::ToBytes;

use super::CasmProgram;
use crate::vm::{
    allocator::{
        stack::{Offset, StackSlice},
        Memory, MemoryAddress,
    },
    casm::operation::OpPrimitive,
    vm::{Executable, RuntimeError},
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
                program.cursor.set(program.cursor.get() + 1);
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = memory.stack.push(*size).map_err(|e| e.into())?;
                program.cursor.set(program.cursor.get() + 1);
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
                let _ = memory.stack.clean().map_err(|e| e.into())?;
                let link = OpPrimitive::get_num8::<u64>(memory)?;

                program.cursor.set(link as usize);
                Ok(())
            }
            StackFrame::Set {
                return_size,
                cursor_offset,
            } => {
                let _ = memory.stack.frame(*return_size).map_err(|e| e.into())?;

                let _ = memory
                    .stack
                    .write(
                        Offset::FZ(-24),
                        &(program.cursor.get() + cursor_offset).to_le_bytes(),
                    )
                    .map_err(|e| e.into())?;
                program.cursor.set(program.cursor.get() + 1);
                Ok(())
            }
            StackFrame::Return { return_size } => {
                let return_data = memory.stack.pop(*return_size).map_err(|e| e.into())?;

                let _ = memory
                    .stack
                    .write(Offset::FB(0), &return_data)
                    .map_err(|e| e.into())?;

                let _ = memory.stack.clean().map_err(|e| e.into())?;

                let link = OpPrimitive::get_num8::<u64>(memory)?;
                // dbg!(link);
                program.cursor.set(link as usize);
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
                    program.cursor.set(program.cursor.get() + 1);
                    Ok(())
                }
                MemoryAddress::Stack { offset } => {
                    let data = memory
                        .stack
                        .read(*offset, *size)
                        .map_err(|err| err.into())?;
                    // Copy data onto stack;
                    let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                    program.cursor.set(program.cursor.get() + 1);
                    Ok(())
                }
            },
            Access::Runtime { size: _ } => {
                let _address = OpPrimitive::get_num8::<u64>(memory)?;
                let _address = {
                    todo!("Convert u64 to memory address by differenting stack pointer and heap pointer");
                };
                program.cursor.set(program.cursor.get() + 1);
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
            .read(self.stack_slice.offset, self.stack_slice.size)
            .map_err(|err| err.into())?;

        match self.address {
            MemoryAddress::Heap => {
                let address = OpPrimitive::get_num8::<u64>(memory)? as usize;
                let _ = memory.heap.write(address, &data).map_err(|e| e.into())?;
            }
            MemoryAddress::Stack { offset, .. } => {
                let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
            }
        };

        program.cursor.set(program.cursor.get() + 1);
        Ok(())
    }
}
