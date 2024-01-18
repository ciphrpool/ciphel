use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub enum LoopStrip {}

impl Executable for LoopStrip {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
