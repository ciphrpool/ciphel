use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

use super::operation::OpPrimitive;

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl Executable for Locate {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        todo!()
    }
}
