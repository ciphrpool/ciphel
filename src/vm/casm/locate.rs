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
use num_traits::ToBytes;

use super::CasmProgram;

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Locate {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, &format!("addr {}", self.address.name()));
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Locate {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let address: u64 = self.address.into();

        let _ = stack.push_with(&address.to_le_bytes())?;
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateOffsetFromStackPointer {
    pub offset: usize,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LocateOffsetFromStackPointer {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, &format!("addr SP[-{}]", self.offset));
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for LocateOffsetFromStackPointer {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let address =
            (stack.top().checked_sub(self.offset)).ok_or(RuntimeError::MemoryViolation)?;
        let address @ MemoryAddress::Stack { .. } = address.try_into()? else {
            return Err(RuntimeError::MemoryViolation);
        };
        let address: u64 = address.into();

        let _ = stack.push_with(&address.to_le_bytes())?;
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateOffset {
    pub offset: usize,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LocateOffset {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm(engine, &format!("offset {}", self.offset));
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for LocateOffset {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LocateIndex {
    pub size: usize,
    pub base_address: Option<MemoryAddress>,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LocateIndex {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self.base_address {
            Some(addr) => stdio.push_casm(
                engine,
                &format!("addr_at_idx {},{}", addr.name(), self.size),
            ),
            None => stdio.push_casm(engine, &format!("addr_at_idx _,{}", self.size)),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for LocateIndex {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum LocateUTF8Char {
    RuntimeNext,
    RuntimeAtIdx { len: Option<usize> },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for LocateUTF8Char {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            LocateUTF8Char::RuntimeNext => stdio.push_casm(engine, "addr_utf8 "),
            LocateUTF8Char::RuntimeAtIdx { .. } => stdio.push_casm(engine, "addr_utf8_at"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            LocateUTF8Char::RuntimeNext => crate::vm::vm::CasmWeight::LOW,
            LocateUTF8Char::RuntimeAtIdx { len } => crate::vm::vm::CasmWeight::MEDIUM,
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for LocateUTF8Char {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        program.incr();
        match self {
            LocateUTF8Char::RuntimeNext => {
                let pointer = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;
                let offset = match pointer {
                    MemoryAddress::Heap { offset } => {
                        let (_, offset) = heap.read_utf8(offset, 1, 1)?;
                        offset
                    }
                    MemoryAddress::Stack { offset } => {
                        todo!()
                    }
                    MemoryAddress::Global { offset } => todo!(),
                };
                let pointer: u64 = pointer.into();
                let _ = stack.push_with(&(pointer + offset as u64).to_le_bytes());

                Ok(())
            }
            LocateUTF8Char::RuntimeAtIdx { len } => {
                /* */
                let idx = OpPrimitive::get_num8::<u64>(stack)?;
                let pointer = OpPrimitive::get_num8::<u64>(stack)?.try_into()?;

                let size = match pointer {
                    MemoryAddress::Heap { offset } => {
                        let len_bytes = heap.read(offset, 8)?;
                        let len_bytes = TryInto::<&[u8; 8]>::try_into(len_bytes.as_slice())
                            .map_err(|_| RuntimeError::Deserialization)?;
                        let len = u64::from_le_bytes(*len_bytes) as usize;

                        let (_, size) = heap.read_utf8(offset + 16, idx as usize, len)?;
                        size
                    }
                    MemoryAddress::Stack { offset } => {
                        let Some(len) = len else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        todo!()
                    }
                    MemoryAddress::Global { offset } => todo!(),
                };

                let pointer: u64 = pointer.into();
                let _ = stack.push_with(&(pointer + size as u64).to_le_bytes());

                Ok(())
            }
        }
    }
}
