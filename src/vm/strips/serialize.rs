use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub enum Serialize {}

impl Executable for Serialize {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
