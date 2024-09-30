use crate::vm::{
    allocator::{stack::STACK_SIZE, MemoryAddress},
    asm::operation::OpPrimitive,
    runtime::RuntimeError,
    scheduler::Executable,
    stdio::StdIO,
};

use super::{data::HexSlice, operation::PopNum};

pub const ALLOC_SIZE_THRESHOLD: usize = STACK_SIZE / 10;

#[derive(Debug, Clone)]
pub enum Alloc {
    Heap {
        size: usize,
    },
    Stack {
        size: usize,
    },
    Global {
        address: MemoryAddress,
        data: Box<[u8]>,
    },
    GlobalFromStack {
        address: MemoryAddress,
        size: usize,
    },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Alloc {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Alloc::Heap { size } => stdio.push_asm(engine, &format!("halloc {size}")),
            Alloc::Stack { size } => stdio.push_asm(engine, &format!("salloc {size}")),
            Alloc::Global { data, .. } => {
                stdio.push_asm(engine, &format!("galloc 0x{}", HexSlice(data.as_ref())))
            }
            Alloc::GlobalFromStack { address, size } => {
                stdio.push_asm(engine, &format!("galloc {}", size))
            }
        }
    }
}

impl crate::vm::AsmWeight for Alloc {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            Alloc::Heap { size } => {
                if *size > ALLOC_SIZE_THRESHOLD {
                    crate::vm::Weight::EXTREME
                } else {
                    crate::vm::Weight::MEDIUM
                }
            }
            Alloc::Stack { size } => {
                if *size > ALLOC_SIZE_THRESHOLD {
                    crate::vm::Weight::EXTREME
                } else {
                    crate::vm::Weight::MEDIUM
                }
            }
            Alloc::Global { data, .. } => {
                if data.len() > ALLOC_SIZE_THRESHOLD {
                    crate::vm::Weight::EXTREME
                } else {
                    crate::vm::Weight::MEDIUM
                }
            }
            Alloc::GlobalFromStack { size, .. } => {
                if *size > ALLOC_SIZE_THRESHOLD {
                    crate::vm::Weight::EXTREME
                } else {
                    crate::vm::Weight::MEDIUM
                }
            }
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Alloc {
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
            Alloc::Heap { size } => {
                let address = heap.alloc(*size)?;
                let address: u64 = address.into(stack);

                let _ = stack.push_with(&address.to_le_bytes())?;
                scheduler.next();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = stack.push(*size)?;
                scheduler.next();
                Ok(())
            }
            Alloc::Global { data, address } => {
                let _ = stack.write_global(*address, &data)?;

                let address: u64 = (*address).into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
                scheduler.next();
                Ok(())
            }
            Alloc::GlobalFromStack { address, size } => {
                let data = stack.pop(*size)?.to_owned();
                let _ = stack.write_global(*address, &data)?;

                let address: u64 = (*address).into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Realloc {
    pub size: Option<usize>,
}
impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Realloc {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self.size {
            Some(n) => stdio.push_asm(engine, &format!("realloc {n}")),
            None => stdio.push_asm(engine, "realloc"),
        }
    }
}
impl crate::vm::AsmWeight for Realloc {
    fn weight(&self) -> crate::vm::Weight {
        if self.size.unwrap_or(0) > ALLOC_SIZE_THRESHOLD {
            crate::vm::Weight::EXTREME
        } else {
            crate::vm::Weight::MEDIUM
        }
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for Realloc {
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
        let size = match self.size {
            Some(size) => size,
            None => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                size as usize
            }
        };

        let heap_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        let address = heap.realloc(heap_address, size)?;
        let address: u64 = address.into(stack);

        let _ = stack.push_with(&address.to_le_bytes())?;
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Free();
impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Free {
    fn name(&self, stdio: &mut StdIO, _program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, "free");
    }
}
impl crate::vm::AsmWeight for Free {}

impl<E: crate::vm::external::Engine> Executable<E> for Free {
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
        let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        heap.free(address)?;
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: Option<usize> },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Access {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Access::Static { address, size } => {
                stdio.push_asm(engine, &format!("load {} {size}", address.name()))
            }
            Access::Runtime { size } => match size {
                Some(n) => stdio.push_asm(engine, &format!("load {n}")),
                None => stdio.push_asm(engine, "load"),
            },
        }
    }
}
impl crate::vm::AsmWeight for Access {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            Access::Static { .. } => crate::vm::Weight::LOW,
            Access::Runtime { .. } => crate::vm::Weight::LOW,
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Access {
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
        let (address, size) = match self {
            Access::Static { address, size } => (*address, *size),
            Access::Runtime { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::pop_num::<u64>(stack)?;
                        size as usize
                    }
                };
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                (address, size)
            }
        };
        match address {
            MemoryAddress::Heap { .. } => {
                let data = heap.read(address, size)?;

                let _ = stack.push_with(&data)?;
                scheduler.next();
                Ok(())
            }
            MemoryAddress::Stack { .. } => {
                let data = stack.read(address, size)?;
                let data = data.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                scheduler.next();
                Ok(())
            }
            MemoryAddress::Frame { .. } => {
                let data = stack.read_in_frame(address, size)?;
                let data = data.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                scheduler.next();
                Ok(())
            }
            MemoryAddress::Global { .. } => {
                let data = stack.read_global(address, size)?.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                scheduler.next();
                Ok(())
            }
        }
    }
}

// #[derive(Debug, Clone)]

// pub struct CheckIndex {
//     pub item_size: usize,
//     pub len: Option<usize>,
// }

// impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for CheckIndex {
//     fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
//         stdio.push_asm(engine, "chidx");
//     }
//     fn weight(&self) -> crate::vm::Weight {
//         crate::vm::Weight::ZERO
//     }
// }

// impl<E: crate::vm::external::Engine> Executable<E> for CheckIndex {
//     fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
//         &self,
//         program: &mut crate::vm::program::Program,
//         stack: &mut Stack,
//         heap: &mut Heap,
//         stdio: &mut StdIO,
//         engine: &mut E,
//         tid: usize,
//     ) -> Result<(), RuntimeError> {
//         let index = OpPrimitive::pop_num::<u64>(stack)?;
//         let address = OpPrimitive::pop_num::<u64>(stack)?;

//         if address < STACK_SIZE as u64 {
//             if let Some(len) = self.len {
//                 if index >= len as u64 {
//                     return Err(RuntimeError::IndexOutOfBound);
//                 }
//                 let item_addr = (address as u64 + index * self.item_size as u64) as u64;
//                 let _ = stack.push_with(&item_addr.to_le_bytes())?;
//             } else {
//                 return Err(RuntimeError::CodeSegmentation);
//             }
//         } else {
//             let len_bytes = heap.read(address as usize, 8)?;
//             let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
//                 .map_err(|_| RuntimeError::Deserialization)?;
//             let len = u64::from_le_bytes(*len_bytes);
//             if index >= len {
//                 return Err(RuntimeError::IndexOutOfBound);
//             }
//             let item_addr = (address as u64 + 16 + index * self.item_size as u64) as u64;
//             let _ = stack.push_with(&item_addr.to_le_bytes())?;
//         }

//         scheduler.next();
//         Ok(())
//     }
// }
