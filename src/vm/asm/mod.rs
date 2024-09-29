use branch::BranchIf;
use std::collections::HashMap;
use ulid::Ulid;

use crate::ast::statements::{Statement, WithLine};

use self::{branch::Label, data::Data};

use super::{
    allocator::{heap::Heap, stack::Stack},
    core,
    program::Program,
    runtime::RuntimeError,
    scheduler::Executable,
    stdio::StdIO,
};
pub mod alloc;
pub mod branch;
pub mod data;
pub mod locate;
mod math_operation;
pub mod mem;
pub mod operation;

#[derive(Debug, Clone)]
pub enum Asm {
    Core(core::CoreAsm),
    Return(branch::Return),
    CloseFrame(branch::CloseFrame),
    Alloc(alloc::Alloc),
    Realloc(alloc::Realloc),
    Free(alloc::Free),
    Mem(mem::Mem),
    Operation(operation::Operation),
    Data(data::Data),
    Access(alloc::Access),
    // AccessIdx(alloc::CheckIndex),
    Locate(locate::Locate),
    OffsetIdx(locate::LocateIndex),
    OffsetSP(locate::LocateOffsetFromStackPointer),
    Offset(locate::LocateOffset),
    If(branch::BranchIf),
    Try(branch::BranchTry),
    Label(branch::Label),
    Call(branch::Call),
    Goto(branch::Goto),
    // Switch(branch::BranchTable),
    Pop(usize),
}

impl<E: crate::vm::external::Engine> Executable<E> for Asm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::runtime::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            Asm::Operation(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Return(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::CloseFrame(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Data(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Access(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            // Asm::AccessIdx(value) => value.execute(program, scheduler, signal_handler, stack, heap, stdio, engine, context),
            Asm::If(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Label(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Call(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Goto(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Alloc(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Mem(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            // Asm::Switch(value) => value.execute(program, scheduler, signal_handler, stack, heap, stdio, engine, context),
            Asm::OffsetIdx(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Offset(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::OffsetSP(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Locate(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Core(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Pop(size) => {
                stack.pop(*size)?;
                scheduler.next();
                Ok(())
            }
            Asm::Realloc(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Free(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Asm::Try(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
        }
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Asm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Asm::Core(value) => value.name(stdio, program, engine),
            Asm::Return(value) => value.name(stdio, program, engine),
            Asm::CloseFrame(value) => value.name(stdio, program, engine),
            Asm::Alloc(value) => value.name(stdio, program, engine),
            Asm::Realloc(value) => value.name(stdio, program, engine),
            Asm::Free(value) => value.name(stdio, program, engine),
            Asm::Mem(value) => value.name(stdio, program, engine),
            Asm::Operation(value) => value.name(stdio, program, engine),
            Asm::Data(value) => value.name(stdio, program, engine),
            Asm::Access(value) => value.name(stdio, program, engine),
            // Asm::AccessIdx(value) => value.name(stdio, program, engine),
            Asm::Offset(value) => value.name(stdio, program, engine),
            Asm::OffsetIdx(value) => value.name(stdio, program, engine),
            Asm::OffsetSP(value) => value.name(stdio, program, engine),
            Asm::Locate(value) => value.name(stdio, program, engine),
            Asm::If(value) => value.name(stdio, program, engine),
            Asm::Label(value) => value.name(stdio, program, engine),
            Asm::Call(value) => value.name(stdio, program, engine),
            Asm::Goto(value) => value.name(stdio, program, engine),
            // Asm::Switch(value) => value.name(stdio, program, engine),
            Asm::Try(value) => value.name(stdio, program, engine),
            Asm::Pop(n) => stdio.push_asm(engine, &format!("pop {n}")),
        }
    }
}
impl crate::vm::AsmWeight for Asm {
    fn weight(&self) -> super::Weight {
        match self {
            Asm::Core(value) => value.weight(),
            Asm::Return(value) => value.weight(),
            Asm::CloseFrame(value) => value.weight(),
            Asm::Alloc(value) => value.weight(),
            Asm::Realloc(value) => value.weight(),
            Asm::Free(value) => value.weight(),
            Asm::Mem(value) => value.weight(),
            Asm::Operation(value) => value.weight(),
            Asm::Data(value) => value.weight(),
            Asm::Access(value) => value.weight(),
            Asm::OffsetSP(value) => value.weight(),
            Asm::Offset(value) => value.weight(),
            Asm::OffsetIdx(value) => value.weight(),
            Asm::Locate(value) => value.weight(),
            Asm::If(value) => value.weight(),
            Asm::Try(value) => value.weight(),
            Asm::Label(value) => value.weight(),
            Asm::Call(value) => value.weight(),
            Asm::Goto(value) => value.weight(),
            Asm::Pop(value) => super::Weight::ZERO,
        }
    }
}
