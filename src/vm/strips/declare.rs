use crate::vm::{
    allocator::{Memory},
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Declare {
    size: usize,
}

impl Executable for Declare {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let _ = memory.stack.push(self.size).map_err(|e| e.into())?;
        Ok(())
    }
}
