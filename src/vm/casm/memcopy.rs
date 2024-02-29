use num_traits::ToBytes;

use super::operation::OpPrimitive;
use super::CasmProgram;
use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{stack::Offset, Memory, MemoryAddress},
        vm::{Executable, RuntimeError},
    },
};
use std::{cell::Cell, os::raw::c_uint};

#[derive(Debug, Clone)]
pub enum MemCopy {
    Clone {
        from: MemoryAddress,
        to: MemoryAddress,
    },
    CloneFromSmartPointer,
    TakeToHeap {
        size: usize,
    },
    TakeToStack {
        size: usize,
    },
}

impl Executable for MemCopy {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            MemCopy::Clone { from: _, to: _ } => todo!(),
            MemCopy::CloneFromSmartPointer => {
                let heap_address = OpPrimitive::get_num8::<u64>(memory)?;
                let data = memory
                    .heap
                    .read(heap_address as usize, 8)
                    .map_err(|e| e.into())?;
                let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let size = u64::from_le_bytes(*data);
                let data = memory
                    .heap
                    .read(heap_address as usize + 16, size as usize)
                    .map_err(|e| e.into())?;
                let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                let _ = memory
                    .stack
                    .push_with(&size.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::TakeToHeap { size } => {
                let heap_address = OpPrimitive::get_num8::<u64>(memory)?;

                let data = memory.stack.pop(*size).map_err(|e| e.into())?;

                let _ = memory
                    .heap
                    .write(heap_address as usize, &data)
                    .map_err(|e| e.into())?;
                let _ = memory
                    .stack
                    .push_with(&heap_address.to_le_bytes())
                    .map_err(|e| e.into())?;
            }
            MemCopy::TakeToStack { size } => {
                let stack_address = OpPrimitive::get_num8::<u64>(memory)?;
                let data = memory.stack.pop(*size).map_err(|e| e.into())?;
                let _ = memory
                    .stack
                    .write(
                        Offset::SB(stack_address as usize),
                        AccessLevel::General,
                        &data,
                    )
                    .map_err(|e| e.into())?;
            }
        }

        program.incr();
        Ok(())
    }
}
