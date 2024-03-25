use super::CasmProgram;
use crate::vm::{
    allocator::{
        stack::{Offset, StackError, STACK_SIZE},
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
