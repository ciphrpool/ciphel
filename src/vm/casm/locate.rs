use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            heap::Heap,
            stack::{Offset, Stack, STACK_SIZE},
            MemoryAddress,
        },
        casm::operation::OpPrimitive,
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};
use num_traits::ToBytes;

use super::CasmProgram;

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Locate {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, &format!("addr {}", self.address.name()));
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Locate {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match &self.address {
            MemoryAddress::Heap { offset } => {
                // let _ = memory.stack.push_with(&data).map_err(|e| e.into())?;
                let data = (STACK_SIZE as u64 + *offset as u64).to_le_bytes();
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            MemoryAddress::Stack { offset, level } => {
                if let Offset::FE(stack_idx, heap_idx) = offset {
                    let heap_address = stack
                        .read(Offset::FP(*stack_idx), *level, 8)
                        .map_err(|err| err.into())?;
                    let data = TryInto::<&[u8; 8]>::try_into(heap_address)
                        .map_err(|_| RuntimeError::Deserialization)?;
                    let heap_address = u64::from_le_bytes(*data);
                    let data = (heap_address + *heap_idx as u64).to_le_bytes();
                    let _ = stack.push_with(&data).map_err(|e| e.into())?;
                    program.incr();
                    return Ok(());
                }
                let offset = stack
                    .compute_absolute_address(*offset, *level)
                    .map_err(|e| e.into())?;
                let data = (offset as u64).to_le_bytes();
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LocateUTF8Char {
    RuntimeNext,
    RuntimeAtIdx { len: Option<usize> },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LocateUTF8Char {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram, engine: &mut G) {
        match self {
            LocateUTF8Char::RuntimeNext => stdio.push_casm(engine, "addr_utf8 "),
            LocateUTF8Char::RuntimeAtIdx { .. } => stdio.push_casm(engine, "addr_utf8_at"),
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for LocateUTF8Char {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        program.incr();
        match self {
            LocateUTF8Char::RuntimeNext => {
                let pointer = OpPrimitive::get_num8::<u64>(stack)?;

                let address = {
                    if pointer < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(pointer as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: pointer as usize,
                        }
                    }
                };
                let offset = match address {
                    MemoryAddress::Heap { offset } => {
                        let (_, offset) = heap.read_utf8(offset, 1, 1).map_err(|err| err.into())?;
                        offset
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let (_, offset) = stack
                            .read_utf8(offset, level, 1, 1)
                            .map_err(|err| err.into())?;
                        offset
                    }
                };

                let _ = stack.push_with(&(pointer + offset as u64).to_le_bytes());

                Ok(())
            }
            LocateUTF8Char::RuntimeAtIdx { len } => {
                /* */
                let idx = OpPrimitive::get_num8::<u64>(stack)?;
                let pointer = OpPrimitive::get_num8::<u64>(stack)?;
                let address = {
                    if pointer < STACK_SIZE as u64 {
                        MemoryAddress::Stack {
                            offset: Offset::SB(pointer as usize),
                            level: AccessLevel::General,
                        }
                    } else {
                        MemoryAddress::Heap {
                            offset: pointer as usize,
                        }
                    }
                };

                let size = match address {
                    MemoryAddress::Heap { offset } => {
                        let len_bytes = heap.read(offset, 8).map_err(|e| e.into())?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes) as usize;

                        let (_, size) = heap
                            .read_utf8(offset + 16, idx as usize, len)
                            .map_err(|err| err.into())?;
                        size
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let Some(len) = len else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        let (_, size) = stack
                            .read_utf8(offset, level, idx as usize, *len)
                            .map_err(|err| err.into())?;
                        size
                    }
                };

                let _ = stack.push_with(&(pointer + size as u64).to_le_bytes());

                Ok(())
            }
        }
    }
}
