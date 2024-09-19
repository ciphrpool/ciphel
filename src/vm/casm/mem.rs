use nom::combinator::into;
use num_traits::{CheckedSub, ToBytes};
use ulid::Ulid;

use super::{
    operation::{OpPrimitive, PopNum},
    CasmProgram,
};

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
    Dup(usize),
    Label(Ulid),
    Store { size: usize, address: MemoryAddress },
    Take { size: usize },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Mem {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Mem::Dup(n) => stdio.push_casm(engine, &format!("dup {n}")),
            Mem::Label(label) => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("dmp_label {label}"))
            }
            Mem::Take { size } => stdio.push_casm(engine, &format!("take {size}")),
            Mem::Store { size, address } => {
                stdio.push_casm(engine, &format!("store {}", address.name()))
            }
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Mem::Dup(_) => crate::vm::vm::CasmWeight::ZERO,
            Mem::Label(_) => crate::vm::vm::CasmWeight::LOW,
            Mem::Take { size } => crate::vm::vm::CasmWeight::LOW,
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
                let Some(idx) = program.get(label) else {
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

        program.incr();
        Ok(())
    }
}
