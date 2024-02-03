use crate::{
    vm::{
        allocator::Memory,
        vm::{Executable, RuntimeError},
    },
};

#[derive(Debug, Clone)]
pub struct Serialized {
    pub data: Vec<u8>,
    //pub data_type: Either<UserType, StaticType>,
}

impl Executable for Serialized {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let offset = memory.stack.top();
        let _ = memory.stack.push(self.data.len()).map_err(|e| e.into())?;
        let _ = memory
            .stack
            .write(offset, &self.data)
            .map_err(|e| e.into())?;

        Ok(())
    }
}
