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
        todo!()
    }
}
