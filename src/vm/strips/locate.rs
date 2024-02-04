use num_traits::ToBytes;

use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl Executable for Locate {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self.address {
            MemoryAddress::Heap => {
                // let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                todo!();
                Ok(())
            }
            MemoryAddress::Stack { offset } => {
                let data = (offset as u64).to_le_bytes();
                let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                Ok(())
            }
        }
    }
}
