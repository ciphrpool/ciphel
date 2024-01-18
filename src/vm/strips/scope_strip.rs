use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct ScopeStrip {}

impl Executable for ScopeStrip {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
