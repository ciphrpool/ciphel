use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Serialized {
    pub data: Vec<u8>,
    //pub data_type: Either<UserType, StaticType>,
}

impl Executable for Serialized {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let _ = memory.stack.push_with(&self.data).map_err(|e| e.into())?;
        Ok(())
    }
}
