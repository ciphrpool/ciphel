use nom::combinator::into;
use num_traits::{CheckedSub, ToBytes};
use ulid::Ulid;

use super::{operation::OpPrimitive, CasmProgram};

use crate::{
    semantic::AccessLevel,
    vm::{
        allocator::{
            heap::Heap,
            stack::{Stack, STACK_SIZE},
            MemoryAddress,
        },
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};

#[derive(Debug, Clone)]
pub enum Mem {
    MemCopy,
    Dup(usize),
    CloneFromSmartPointer(usize),
    LabelOffset(Ulid),
    TakeToHeap { size: usize },
    TakeToStack { size: usize },
    Store { size: usize, address: usize },
    Take { size: usize },
    TakeUTF8Char,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Mem {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Mem::MemCopy => stdio.push_casm(engine, "memcpy"),
            Mem::Dup(n) => stdio.push_casm(engine, &format!("dup {n}")),
            Mem::CloneFromSmartPointer(n) => stdio.push_casm(engine, &format!("clone {n}")),
            Mem::LabelOffset(label) => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("dmp_label {label}"))
            }
            Mem::TakeToHeap { size } => stdio.push_casm(engine, &format!("htake {size}")),
            Mem::TakeToStack { size } => stdio.push_casm(engine, &format!("stake {size}")),
            Mem::Take { size } => stdio.push_casm(engine, &format!("take {size}")),
            Mem::TakeUTF8Char => stdio.push_casm(engine, "take_utf8_char"),
            Mem::Store { size, address } => stdio.push_casm(engine, "store"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Mem::MemCopy => crate::vm::vm::CasmWeight::MEDIUM,
            Mem::Dup(_) => crate::vm::vm::CasmWeight::ZERO,
            Mem::CloneFromSmartPointer(_) => crate::vm::vm::CasmWeight::HIGH,
            Mem::LabelOffset(_) => crate::vm::vm::CasmWeight::LOW,
            Mem::TakeToHeap { size } => crate::vm::vm::CasmWeight::LOW,
            Mem::TakeToStack { size } => crate::vm::vm::CasmWeight::LOW,
            Mem::Take { size } => crate::vm::vm::CasmWeight::LOW,
            Mem::TakeUTF8Char => crate::vm::vm::CasmWeight::LOW,
            Mem::Store { size, address } => crate::vm::vm::CasmWeight::ZERO,
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Mem {
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
            Mem::CloneFromSmartPointer(size) => {
                let heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                let data = heap.read(heap_address as usize, 8)?;
                let data = TryInto::<&[u8; 8]>::try_into(data.as_slice())
                    .map_err(|_| RuntimeError::Deserialization)?;
                let length = u64::from_le_bytes(*data);

                let data = heap.read(heap_address as usize + 16, size * length as usize)?;

                let _ = stack.push_with(&data)?;
                let _ = stack.push_with(&length.to_le_bytes())?;
            }
            Mem::TakeToHeap { size } => {
                let heap_address = OpPrimitive::get_num8::<u64>(stack)?;

                let data = stack.pop(*size)?;
                let _ = heap.write(heap_address as usize, &data.to_vec())?;
                let _ = stack.push_with(&heap_address.to_le_bytes())?;
            }
            Mem::Take { size } => {
                let address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                match address {
                    MemoryAddress::Heap { offset } => {
                        let data = stack.pop(*size)?.to_owned();
                        let _ = stack.write(offset, &data)?;
                    }
                    MemoryAddress::Stack { offset } => {
                        let data = stack.pop(*size)?;
                        let _ = heap.write(offset, &data.to_vec())?;
                        let _ = stack.push_with(&offset.to_le_bytes())?;
                    }
                    MemoryAddress::Global { offset } => todo!(),
                }
            }
            Mem::TakeUTF8Char => {
                let address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                let value = OpPrimitive::get_char(stack)?;
                match address {
                    MemoryAddress::Heap { offset } => {
                        let _ = heap.write(offset, &value.to_string().as_bytes().to_vec())?;
                        let _ = stack.push_with(&offset.to_le_bytes())?;
                    }
                    MemoryAddress::Stack { offset } => {
                        let _ = stack.write(offset, &value.to_string().as_bytes())?;
                    }
                    MemoryAddress::Global { offset } => todo!(),
                }
            }
            Mem::TakeToStack { size } => {
                let stack_address = OpPrimitive::get_num8::<u64>(stack)?;
                let data = stack.pop(*size)?.to_owned();
                let _ = stack.write(stack_address as usize, &data)?;
            }
            Mem::Dup(size) => {
                let data = stack
                    .read(stack.top().checked_sub(*size).unwrap_or(0), *size)?
                    .to_owned();
                let _ = stack.push_with(&data)?;
            }
            Mem::LabelOffset(label) => {
                let Some(idx) = program.get(label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                let idx = idx as u64;
                let _ = stack.push_with(&idx.to_le_bytes())?;
            }
            Mem::MemCopy => {
                let size = OpPrimitive::get_num8::<u64>(stack)? as usize;
                let from_address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                let to_address = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;

                let from_data = match from_address {
                    MemoryAddress::Heap { offset } => {
                        // let address = OpPrimitive::get_num8::<u64>(stack)? as usize;
                        let bytes = heap.read(offset, size)?;
                        bytes
                    }
                    MemoryAddress::Stack { offset } => {
                        let bytes = stack.read(offset, size)?;
                        bytes.to_vec()
                    }
                    MemoryAddress::Global { offset } => todo!(),
                };
                match to_address {
                    MemoryAddress::Heap { offset } => {
                        let _ = heap.write(offset, &from_data)?;
                    }
                    MemoryAddress::Stack { offset } => {
                        let _ = stack.write(offset, &from_data)?;
                    }
                    MemoryAddress::Global { offset } => todo!(),
                }
            }
            Mem::Store { size, address } => todo!(),
        };

        program.incr();
        Ok(())
    }
}
