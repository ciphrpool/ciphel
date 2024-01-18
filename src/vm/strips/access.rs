use crate::vm::{
    allocator::{Memory, MemoryAddress},
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Access {
    address: MemoryAddress,
}

impl Executable for Access {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        match self.address {
            MemoryAddress::Heap {
                address,
                offset,
                size,
            } => {
                let _data = memory
                    .heap
                    .read(address, offset, size)
                    .map_err(|err| err.into())?;
                todo!("Copy data onto stack");
                Ok(())
            }
            MemoryAddress::Stack { offset, size } => {
                let _data = memory.stack.read(offset, size).map_err(|err| err.into())?;

                todo!("Copy data onto stack");
                Ok(())
            }
        }
    }
}
