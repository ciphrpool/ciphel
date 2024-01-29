use crate::vm::{
    allocator::{stack::StackSlice, Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

use super::operation::OpPrimitive;

#[derive(Debug, Clone)]
pub struct Assign {
    address: MemoryAddress,
    stack_slice: StackSlice,
}

impl Executable for Assign {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
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

        Ok(())
    }
}
