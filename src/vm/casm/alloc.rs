use std::sync::atomic::Ordering;

use num_traits::ToBytes;
use ulid::Ulid;

use crate::vm::{
    allocator::{
        heap::Heap,
        stack::{Stack, STACK_SIZE},
        MemoryAddress,
    },
    casm::operation::OpPrimitive,
    stdio::StdIO,
    vm::{CasmMetadata, Executable, RuntimeError},
};

use super::{data::HexSlice, operation::PopNum, CasmProgram};

pub const ALLOC_SIZE_THRESHOLD: usize = STACK_SIZE / 10;

#[derive(Debug, Clone)]
pub enum Alloc {
    Heap {
        size: Option<usize>,
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

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Alloc {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Alloc::Heap { size } => match size {
                None => stdio.push_casm(engine, "halloc"),
                Some(n) => stdio.push_casm(engine, &format!("halloc {n}")),
            },
            Alloc::Stack { size } => stdio.push_casm(engine, &format!("salloc {size}")),
            Alloc::Global { data, .. } => {
                stdio.push_casm(engine, &format!("galloc 0x{}", HexSlice(data.as_ref())))
            }
            Alloc::GlobalFromStack { address, size } => {
                stdio.push_casm(engine, &format!("galloc {}", size))
            }
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Alloc::Heap { size } => {
                if size.unwrap_or(0) > ALLOC_SIZE_THRESHOLD {
                    crate::vm::vm::CasmWeight::EXTREME
                } else {
                    crate::vm::vm::CasmWeight::MEDIUM
                }
            }
            Alloc::Stack { size } => {
                if *size > ALLOC_SIZE_THRESHOLD {
                    crate::vm::vm::CasmWeight::EXTREME
                } else {
                    crate::vm::vm::CasmWeight::MEDIUM
                }
            }
            Alloc::Global { data, .. } => {
                if data.len() > ALLOC_SIZE_THRESHOLD {
                    crate::vm::vm::CasmWeight::EXTREME
                } else {
                    crate::vm::vm::CasmWeight::MEDIUM
                }
            }
            Alloc::GlobalFromStack { address, size } => {
                if *size > ALLOC_SIZE_THRESHOLD {
                    crate::vm::vm::CasmWeight::EXTREME
                } else {
                    crate::vm::vm::CasmWeight::MEDIUM
                }
            }
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for Alloc {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            Alloc::Heap { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => OpPrimitive::pop_num::<u64>(stack)? as usize,
                };
                let address = heap.alloc(size)?;
                let address: u64 = address.into(stack);

                let _ = stack.push_with(&address.to_le_bytes())?;
                program.incr();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = stack.push(*size)?;
                program.incr();
                Ok(())
            }
            Alloc::Global { data, address } => {
                let _ = stack.write_global(*address, &data)?;

                let address: u64 = (*address).into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
                program.incr();
                Ok(())
            }
            Alloc::GlobalFromStack { address, size } => {
                let data = stack.pop(*size)?.to_owned();
                let _ = stack.write_global(*address, &data)?;

                let address: u64 = (*address).into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
                program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Realloc {
    pub size: Option<usize>,
}
impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Realloc {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self.size {
            Some(n) => stdio.push_casm(engine, &format!("realloc {n}")),
            None => stdio.push_casm(engine, "realloc"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        if self.size.unwrap_or(0) > ALLOC_SIZE_THRESHOLD {
            crate::vm::vm::CasmWeight::EXTREME
        } else {
            crate::vm::vm::CasmWeight::MEDIUM
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for Realloc {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
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
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Free();
impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Free {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, "free");
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for Free {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        heap.free(address)?;
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: Option<usize> },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Access {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Access::Static { address, size } => {
                stdio.push_casm(engine, &format!("load {} {size}", address.name()))
            }
            Access::Runtime { size } => match size {
                Some(n) => stdio.push_casm(engine, &format!("load {n}")),
                None => stdio.push_casm(engine, "load"),
            },
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Access::Static { address, size } => crate::vm::vm::CasmWeight::LOW,
            Access::Runtime { size } => crate::vm::vm::CasmWeight::LOW,
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Access {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
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
                program.incr();
                Ok(())
            }
            MemoryAddress::Stack { .. } => {
                let data = stack.read(address, size)?;
                let data = data.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Frame { .. } => {
                let data = stack.read_in_frame(address, size)?;
                let data = data.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Global { .. } => {
                let data = stack.read_global(address, size)?.to_vec();

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                program.incr();
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

// impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for CheckIndex {
//     fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
//         stdio.push_casm(engine, "chidx");
//     }
//     fn weight(&self) -> crate::vm::vm::CasmWeight {
//         crate::vm::vm::CasmWeight::ZERO
//     }
// }

// impl<G: crate::GameEngineStaticFn> Executable<G> for CheckIndex {
//     fn execute(
//         &self,
//         program: &mut CasmProgram,
//         stack: &mut Stack,
//         heap: &mut Heap,
//         stdio: &mut StdIO,
//         engine: &mut G,
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

//         program.incr();
//         Ok(())
//     }
// }
