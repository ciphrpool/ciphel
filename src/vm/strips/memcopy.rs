use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

use super::operation::OpPrimitive;

#[derive(Debug, Clone)]
pub enum MemCopy {
    Clone {
        from: MemoryAddress,
        to: MemoryAddress,
    },
    Take {
        offset: usize,
        size: usize,
    },
}

impl Executable for MemCopy {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            MemCopy::Clone { from, to } => todo!(),
            MemCopy::Take { offset, size } => {
                let heap_address = OpPrimitive::get_num8::<u64>(memory)?;
                let data = memory.stack.pop(*size).map_err(|e| e.into())?;
                let _ = memory
                    .heap
                    .write(heap_address as usize, &data)
                    .map_err(|e| e.into())?;
            }
        }

        Ok(())
    }
}
