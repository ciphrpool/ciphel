use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct MatchStrip {}

impl Executable for MatchStrip {
    fn execute(&self, _memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
