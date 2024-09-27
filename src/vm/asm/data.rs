use std::fmt;

use ulid::Ulid;

use crate::vm::{
    allocator::{heap::Heap, stack::Stack},
    program::Program,
    runtime::RuntimeError,
    scheduler_v2::Executable,
    stdio::StdIO,
};

use super::alloc::ALLOC_SIZE_THRESHOLD;
pub struct HexSlice<'a>(pub &'a [u8]);

impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{:0>2x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    Serialized { data: Box<[u8]> },
    Dump { data: Box<[Box<[u8]>]> },
    Table { data: Box<[Ulid]> },
    // Get { label: Ulid, idx: Option<usize> },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Data {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Data::Serialized { data } => {
                stdio.push_asm(engine, &format!("dmp 0x{}", HexSlice(data.as_ref())))
            }
            Data::Dump { data } => {
                let arr: Vec<String> = data.iter().map(|e| format!("0x{}", HexSlice(e))).collect();
                let arr = arr.join(", ");
                stdio.push_asm(engine, &format!("data {}", arr))
            }
            Data::Table { data } => {
                let arr: Vec<String> = data
                    .iter()
                    .map(|e| {
                        let label = program
                            .get_label_name(e)
                            .unwrap_or("".to_string().into())
                            .to_string();
                        label.to_string()
                    })
                    .collect();
                let arr = arr.join(", ");
                stdio.push_asm(engine, &format!("labels {}", arr))
            }
        }
    }
}
impl crate::vm::AsmWeight for Data {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            Data::Serialized { data } => {
                if data.len() > ALLOC_SIZE_THRESHOLD {
                    crate::vm::Weight::EXTREME
                } else {
                    crate::vm::Weight::LOW
                }
            }
            Data::Dump { data } => crate::vm::Weight::ZERO,
            Data::Table { data } => crate::vm::Weight::ZERO,
        }
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for Data {
    fn execute<P: crate::vm::scheduler_v2::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler_v2::Scheduler<P>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler_v2::ExecutionContext,
    ) -> Result<(), RuntimeError> {
        match self {
            Data::Serialized { data } => {
                let _ = stack.push_with(&data)?;
                scheduler.next();
            }
            Data::Dump { data } => scheduler.next(),
            Data::Table { data } => scheduler.next(),
        }
        Ok(())
    }
}
