
use crate::vm::{
    scheduler::Thread,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Serialized {
    pub data: Vec<u8>,
    //pub data_type: EType,
}

impl Executable for Serialized {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let _ = thread
            .env
            .stack
            .push_with(&self.data)
            .map_err(|e| e.into())?;
        thread.env.program.incr();
        Ok(())
    }
}
