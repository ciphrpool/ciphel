use num_traits::ToBytes;
use ulid::Ulid;

use super::operation::{OpPrimitive, PopNum};

use crate::vm::{
    allocator::MemoryAddress, runtime::RuntimeError, scheduler::Executable, stdio::StdIO,
};

#[derive(Debug, Clone)]
pub enum Mem {
    Dup(usize),
    Label(Ulid),
    Store { size: usize, address: MemoryAddress },
    Take { size: usize },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Mem {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Mem::Dup(n) => stdio.push_asm(engine, &format!("dup {n}")),
            Mem::Label(label) => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_asm(engine, &format!("dmp_label {label}"))
            }
            Mem::Take { size } => stdio.push_asm(engine, &format!("take {size}")),
            Mem::Store { size, address } => {
                stdio.push_asm(engine, &format!("store {} {}", address.name(), size))
            }
        }
    }
}
impl crate::vm::AsmWeight for Mem {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            Mem::Dup(_) => crate::vm::Weight::ZERO,
            Mem::Label(_) => crate::vm::Weight::LOW,
            Mem::Take { size } => crate::vm::Weight::LOW,
            Mem::Store { size, address } => crate::vm::Weight::ZERO,
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Mem {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            Mem::Take { size } => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                match address {
                    MemoryAddress::Stack { offset } => {
                        let data = stack.pop(*size)?.to_owned();
                        let _ = stack.write(address, &data)?;
                    }
                    MemoryAddress::Heap { offset } => {
                        let data = stack.pop(*size)?;
                        let _ = heap.write(address, &data.to_vec())?;
                        let heap_address: u64 = address.into(stack);
                        let _ = stack.push_with(&heap_address.to_le_bytes())?;
                    }
                    MemoryAddress::Global { offset } => {
                        let data = stack.pop(*size)?.to_vec();
                        let _ = stack.write_global(address, &data)?;
                    }
                    MemoryAddress::Frame { offset } => {
                        let data = stack.pop(*size)?.to_owned();
                        let _ = stack.write_in_frame(address, &data)?;
                    }
                }
                let address: u64 = address.into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            Mem::Dup(size) => {
                let address = MemoryAddress::Stack {
                    offset: stack.top(),
                }
                .sub(*size);
                let data = stack.read(address, *size)?.to_owned();
                let _ = stack.push_with(&data)?;
            }
            Mem::Label(label) => {
                let Some(idx) = program.get_cursor_from_label(label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                let idx = idx as u64;
                let _ = stack.push_with(&idx.to_le_bytes())?;
            }
            Mem::Store { size, address } => {
                let data = stack.pop(*size)?.to_vec();

                match address {
                    MemoryAddress::Heap { .. } => {
                        let _ = heap.write(*address, &data)?;
                    }
                    MemoryAddress::Stack { .. } => {
                        let _ = stack.write(*address, &data)?;
                    }
                    MemoryAddress::Global { .. } => {
                        let _ = stack.write_global(*address, &data)?;
                    }
                    MemoryAddress::Frame { .. } => {
                        let _ = stack.write_in_frame(*address, &data)?;
                    }
                }
            }
        };

        scheduler.next();
        Ok(())
    }
}
