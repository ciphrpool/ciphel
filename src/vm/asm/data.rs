use std::fmt;

use ulid::Ulid;

use crate::vm::{runtime::RuntimeError, scheduler::Executable, stdio::StdIO};

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
    // Get { label: Ulid, idx: Option<usize> },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Data {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Data::Serialized { data } => {
                stdio.push_asm(engine, &format!("dmp 0x{}", HexSlice(data.as_ref())))
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
        }
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for Data {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            Data::Serialized { data } => {
                let _ = stack.push_with(&data)?;
                scheduler.next();
            }
        }
        Ok(())
    }
}
