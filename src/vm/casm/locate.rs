use super::CasmProgram;
use crate::vm::{
    allocator::{
        stack::{Offset, StackError},
        Memory, MemoryAddress,
    },
    vm::{Executable, RuntimeError},
};
use num_traits::ToBytes;
use std::{cell::Cell, mem};

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl Executable for Locate {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self.address {
            MemoryAddress::Heap => {
                // let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                todo!();
                program.cursor.set(program.cursor.get() + 1);
                Ok(())
            }
            MemoryAddress::Stack { offset } => {
                let offset = match offset {
                    Offset::SB(offset) => offset,
                    Offset::ST(offset) => {
                        if offset < 0 {
                            memory
                                .stack
                                .top()
                                .checked_sub(offset as usize)
                                .ok_or(RuntimeError::Default)?
                        } else {
                            memory.stack.top() + (offset as usize)
                        }
                    }
                    Offset::FB(offset) => {
                        let bottom = memory.stack.frame_bottom();
                        bottom + offset
                    }
                    Offset::FZ(offset) => {
                        let zero = memory.stack.frame_zero();
                        if offset < 0 {
                            zero.checked_sub(offset as usize)
                                .ok_or(RuntimeError::Default)?
                        } else {
                            zero + (offset as usize)
                        }
                    }
                };
                let data = (offset as u64).to_le_bytes();
                let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                program.cursor.set(program.cursor.get() + 1);
                Ok(())
            }
        }
    }
}
