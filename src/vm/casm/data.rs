use ulid::Ulid;

use crate::vm::{
    allocator::{heap::Heap, stack::Stack},
    stdio::StdIO,
    vm::{Executable, RuntimeError},
};

use super::CasmProgram;

#[derive(Debug, Clone)]
pub enum Data {
    Serialized { data: Box<[u8]> },
    Dump { data: Box<[Box<[u8]>]> },
    Table { data: Box<[Ulid]> },
    // Get { label: Ulid, idx: Option<usize> },
}

impl Executable for Data {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            Data::Serialized { data } => {
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
            }
            Data::Dump { data } => program.incr(),
            Data::Table { data } => program.incr(),
            // Data::Get { label, idx } => todo!(),
        }
        Ok(())
    }
}
