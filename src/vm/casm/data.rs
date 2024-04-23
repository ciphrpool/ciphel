use ulid::Ulid;

use crate::vm::{
    scheduler::Thread,
    vm::{Executable, RuntimeError},
};

#[derive(Debug, Clone)]
pub enum Data {
    Serialized { data: Box<[u8]> },
    Dump { data: Box<[Box<[u8]>]> },
    Table { data: Box<[Ulid]> },
    // Get { label: Ulid, idx: Option<usize> },
}

impl Executable for Data {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            Data::Serialized { data } => {
                let _ = thread.env.stack.push_with(&data).map_err(|e| e.into())?;
                thread.env.program.incr();
            }
            Data::Dump { data } => thread.env.program.incr(),
            Data::Table { data } => thread.env.program.incr(),
            // Data::Get { label, idx } => todo!(),
        }
        Ok(())
    }
}
