use super::CasmProgram;
use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};
use std::cell::Cell;
#[derive(Debug, Clone)]
pub struct Serialized {
    pub data: Vec<u8>,
    //pub data_type: EType,
}

impl Executable for Serialized {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let _ = memory.stack.push_with(&self.data).map_err(|e| e.into())?;
        program.cursor.set(program.cursor.get() + 1);
        Ok(())
    }
}
