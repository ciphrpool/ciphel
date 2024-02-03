use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Alloc {
    pub size: usize,
}

impl Executable for Alloc {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let address = memory.heap.alloc(self.size).map_err(|e| e.into())?;
        let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;
        let offset = memory.stack.top();
        let data = (address as u64).to_le_bytes().to_vec();
        let _ = memory.stack.push(data.len()).map_err(|e| e.into())?;
        let _ = memory.stack.write(offset, &data).map_err(|e| e.into())?;
        Ok(())
    }
}
