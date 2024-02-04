use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub enum Alloc {
    Heap { size: usize },
    Stack { size: usize },
}

impl Executable for Alloc {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self {
            Alloc::Heap { size } => {
                let address = memory.heap.alloc(*size).map_err(|e| e.into())?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
                let data = (address as u64).to_le_bytes().to_vec();
                let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = memory.stack.push(*size).map_err(|e| e.into())?;
                Ok(())
            }
        }
    }
}
