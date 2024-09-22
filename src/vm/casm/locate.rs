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
use num_traits::ToBytes;

use super::{
    operation::{GetNumFrom, PopNum},
    CasmProgram,
};

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
        let address: u64 = self.address.into(stack);

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

        let address = MemoryAddress::Stack { offset: address };

        let address: u64 = address.into(stack);

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
        let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        let new_address = address.add(self.offset);
        let new_address: u64 = new_address.into(stack);

        let _ = stack.push_with(&new_address.to_le_bytes())?;
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateIndex {
    pub size: usize,
    pub base_address: Option<MemoryAddress>,
    pub offset: Option<usize>,
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
        let (mut address, index) = match self.base_address {
            Some(address) => {
                let index = OpPrimitive::pop_num::<u64>(stack)?;
                let address: MemoryAddress =
                    OpPrimitive::get_num_from::<u64>(address, stack, heap)?.try_into()?;
                (address, index)
            }
            None => {
                let index = OpPrimitive::pop_num::<u64>(stack)?;
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                (address, index)
            }
        };

        if let Some(offset) = self.offset {
            address = address.add(offset);
        }

        let new_address = address.add(self.size * (index as usize));
        let new_address: u64 = new_address.into(stack);

        let _ = stack.push_with(&new_address.to_le_bytes())?;
        program.incr();
        Ok(())
    }
}
