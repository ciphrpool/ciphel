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
        vm::{Executable, RuntimeError},
    },
};
use num_traits::ToBytes;

use super::CasmProgram;

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl Executable for Locate {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
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
pub enum LocateNextUTF8Char {
    RuntimeNext,
    RuntimeAtIdx,
}

impl Executable for LocateNextUTF8Char {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        program.incr();
        match self {
            LocateNextUTF8Char::RuntimeNext => {
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
                        let (_, offset) = heap.read_utf8(offset, 1).map_err(|err| err.into())?;
                        offset
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let (_, offset) = stack
                            .read_utf8(offset, level, 1)
                            .map_err(|err| err.into())?;
                        offset
                    }
                };

                let _ = stack.push_with(&(pointer + offset as u64).to_le_bytes());

                Ok(())
            }
            LocateNextUTF8Char::RuntimeAtIdx => {
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
                        let (_, size) = heap
                            .read_utf8(offset, idx as usize)
                            .map_err(|err| err.into())?;
                        size
                    }
                    MemoryAddress::Stack { offset, level } => {
                        let (_, size) = stack
                            .read_utf8(offset, level, idx as usize)
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
