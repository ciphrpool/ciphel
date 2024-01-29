use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

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
        todo!()
    }
}
