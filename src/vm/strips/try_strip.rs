use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct TryStrip {}

impl Executable for TryStrip {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
