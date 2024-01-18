use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

use super::Strips;

#[derive(Debug, Clone)]
pub struct IfSrip {
    boolean_expr: MemoryAddress,
    then_strips: Strips,
    else_strips: Strips,
}

impl Executable for IfSrip {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let data = match self.boolean_expr {
            MemoryAddress::Heap {
                address,
                offset,
                size,
            } => memory
                .heap
                .read(address, offset, size)
                .map_err(|e| e.into())?,
            MemoryAddress::Stack { offset, size } => {
                memory.stack.read(offset, size).map_err(|e| e.into())?
            }
        };

        let condition = data.first().map_or(false, |byte| *byte != 0);
        if condition {
            self.then_strips.execute(memory)?;
        } else {
            self.else_strips.execute(memory)?;
        }
        Ok(())
    }
}
