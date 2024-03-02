use super::CasmProgram;
use crate::vm::{
    allocator::{
        stack::{Offset, StackError},
        Memory, MemoryAddress,
    },
    scheduler::Thread,
    vm::{Executable, RuntimeError},
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
            MemoryAddress::Heap => {
                // let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                todo!();
                thread.env.program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset, level } => {
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
