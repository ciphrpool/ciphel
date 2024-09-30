use crate::{
    semantic::scope::static_types::POINTER_SIZE,
    vm::{
        allocator::MemoryAddress, asm::operation::OpPrimitive, runtime::RuntimeError,
        scheduler::Executable, stdio::StdIO,
    },
};
use num_traits::ToBytes;

use super::operation::{GetNumFrom, PopNum};

#[derive(Debug, Clone)]
pub struct Locate {
    pub address: MemoryAddress,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Locate {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, &format!("addr {}", self.address.name()));
    }
}
impl crate::vm::AsmWeight for Locate {}

impl<E: crate::vm::external::Engine> Executable<E> for Locate {
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
        let address: u64 = self.address.into(stack);

        let _ = stack.push_with(&address.to_le_bytes())?;
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateOffsetFromStackPointer {
    pub offset: usize,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for LocateOffsetFromStackPointer {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, &format!("addr SP[-{}]", self.offset));
    }
}
impl crate::vm::AsmWeight for LocateOffsetFromStackPointer {}
impl<E: crate::vm::external::Engine> Executable<E> for LocateOffsetFromStackPointer {
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
        let address =
            (stack.top().checked_sub(self.offset)).ok_or(RuntimeError::MemoryViolation)?;

        let address = MemoryAddress::Stack { offset: address };

        let address: u64 = address.into(stack);

        let _ = stack.push_with(&address.to_le_bytes())?;
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateOffset {
    pub offset: usize,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for LocateOffset {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, &format!("offset {}", self.offset));
    }
}
impl crate::vm::AsmWeight for LocateOffset {}

impl<E: crate::vm::external::Engine> Executable<E> for LocateOffset {
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
        let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
        let new_address = address.add(self.offset);
        let new_address: u64 = new_address.into(stack);

        let _ = stack.push_with(&new_address.to_le_bytes())?;
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LocateIndex {
    pub len: Option<usize>,
    pub size: usize,
    pub base_address: Option<MemoryAddress>,
    pub offset: Option<usize>,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for LocateIndex {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self.base_address {
            Some(addr) => stdio.push_asm(
                engine,
                &format!("addr_at_idx {},{}", addr.name(), self.size),
            ),
            None => stdio.push_asm(engine, &format!("addr_at_idx _,{}", self.size)),
        }
    }
}
impl crate::vm::AsmWeight for LocateIndex {}

impl<E: crate::vm::external::Engine> Executable<E> for LocateIndex {
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

        if let Some(len) = self.len {
            if index >= len as u64 {
                return Err(RuntimeError::IndexOutOfBound);
            }
        } else {
            let len =
                OpPrimitive::get_num_from::<u64>(address.add(POINTER_SIZE), stack, heap)? as usize;
            if index >= len as u64 {
                return Err(RuntimeError::IndexOutOfBound);
            }
        }

        if let Some(offset) = self.offset {
            address = address.add(offset);
        }

        let new_address = address.add(self.size * (index as usize));
        let new_address: u64 = new_address.into(stack);

        let _ = stack.push_with(&new_address.to_le_bytes())?;
        scheduler.next();
        Ok(())
    }
}
