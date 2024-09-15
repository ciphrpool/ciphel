use std::sync::atomic::Ordering;

use num_traits::ToBytes;
use ulid::Ulid;

use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            heap::Heap,
            stack::{Stack, STACK_SIZE},
            MemoryAddress,
        },
        casm::operation::OpPrimitive,
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};

use super::CasmProgram;

pub const ALLOC_SIZE_THRESHOLD: usize = STACK_SIZE / 10;

#[derive(Debug, Clone)]
pub enum Alloc {
    Heap { size: Option<usize> },
    Stack { size: usize },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Alloc {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Alloc::Heap { size } => match size {
                None => stdio.push_casm(engine, "halloc"),
                Some(n) => stdio.push_casm(engine, &format!("halloc {n}")),
            },
            Alloc::Stack { size } => stdio.push_casm(engine, &format!("salloc {size}")),
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
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(stack)?;
                        size as usize
                    }
                };
                let address = heap.alloc(size)?;
                let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

                let data = (address as u64).to_le_bytes().to_vec();
                let _ = stack.push_with(&data)?;
                program.incr();
                Ok(())
            }
            Alloc::Stack { size } => {
                let _ = stack.push(*size)?;
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
                let size = OpPrimitive::get_num8::<u64>(stack)?;
                size as usize
            }
        };

        let heap_address = OpPrimitive::get_num8::<u64>(stack)? - 8;
        let address = heap.realloc(heap_address as usize, size)?;
        let address = address + 8 /* IMPORTANT : Offset the heap pointer to the start of the allocated block */;

        let data = (address as u64).to_le_bytes().to_vec();
        let _ = stack.push_with(&data)?;
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
        let address = OpPrimitive::get_num8::<u64>(stack)? - 8;
        if address < STACK_SIZE as u64 {
            return Err(RuntimeError::StackError(
                crate::vm::allocator::stack::StackError::WriteError,
            ));
        } else {
            heap.free(address as usize)?;
        }
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Access {
    Static { address: MemoryAddress, size: usize },
    Runtime { size: Option<usize> },
    RuntimeCharUTF8,
    RuntimeCharUTF8AtIdx { len: Option<usize> },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Access {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Access::Static { address, size } => {
                stdio.push_casm(engine, &format!("ld {} {size}", address.name()))
            }
            Access::Runtime { size } => match size {
                Some(n) => stdio.push_casm(engine, &format!("ld {n}")),
                None => stdio.push_casm(engine, "ld"),
            },
            Access::RuntimeCharUTF8 => stdio.push_casm(engine, "ld_utf8"),
            Access::RuntimeCharUTF8AtIdx { .. } => stdio.push_casm(engine, "ld_utf8_at"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Access::Static { address, size } => crate::vm::vm::CasmWeight::LOW,
            Access::Runtime { size } => crate::vm::vm::CasmWeight::LOW,
            Access::RuntimeCharUTF8 => crate::vm::vm::CasmWeight::LOW,
            Access::RuntimeCharUTF8AtIdx { len } => crate::vm::vm::CasmWeight::MEDIUM,
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
        let index = match self {
            Access::RuntimeCharUTF8AtIdx { .. } => Some(OpPrimitive::get_num8::<u64>(stack)?),
            _ => None,
        };
        let (address, size) = match self {
            Access::Static { address, size } => (*address, *size),
            Access::RuntimeCharUTF8 | Access::RuntimeCharUTF8AtIdx { .. } => {
                let address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                (address, 1)
            }
            Access::Runtime { size } => {
                let size = match size {
                    Some(size) => *size,
                    None => {
                        let size = OpPrimitive::get_num8::<u64>(stack)?;
                        size as usize
                    }
                };
                let address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                (address, size)
            }
        };
        match address {
            MemoryAddress::Heap { offset } => {
                // let address = OpPrimitive::get_num8::<u64>(stack)? as usize;

                let data = match self {
                    Access::RuntimeCharUTF8 => {
                        todo!()
                    }
                    Access::RuntimeCharUTF8AtIdx { .. } => {
                        todo!()
                    }
                    _ => {
                        let bytes = heap.read(offset, size)?;
                        bytes
                    }
                };

                let _ = stack.push_with(&data)?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset } => {
                let data = match self {
                    Access::RuntimeCharUTF8 => {
                        todo!()
                    }
                    Access::RuntimeCharUTF8AtIdx { len } => {
                        todo!()
                    }
                    _ => {
                        let bytes = stack.read(offset, size)?;
                        bytes.to_vec()
                    }
                };

                // Copy data onto stack;
                let _ = stack.push_with(&data)?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Global { offset } => todo!(),
        }
    }
}

#[derive(Debug, Clone)]

pub struct CheckIndex {
    pub item_size: usize,
    pub len: Option<usize>,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for CheckIndex {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, "chidx");
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for CheckIndex {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let index = OpPrimitive::get_num8::<u64>(stack)?;
        let address = OpPrimitive::get_num8::<u64>(stack)?;

        if address < STACK_SIZE as u64 {
            if let Some(len) = self.len {
                if index >= len as u64 {
                    return Err(RuntimeError::IndexOutOfBound);
                }
                let item_addr = (address as u64 + index * self.item_size as u64) as u64;
                let _ = stack.push_with(&item_addr.to_le_bytes())?;
            } else {
                return Err(RuntimeError::CodeSegmentation);
            }
        } else {
            let len_bytes = heap.read(address as usize, 8)?;
            let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                .map_err(|_| RuntimeError::Deserialization)?;
            let len = u64::from_le_bytes(*len_bytes);
            if index >= len {
                return Err(RuntimeError::IndexOutOfBound);
            }
            let item_addr = (address as u64 + 16 + index * self.item_size as u64) as u64;
            let _ = stack.push_with(&item_addr.to_le_bytes())?;
        }

        program.incr();
        Ok(())
    }
}
