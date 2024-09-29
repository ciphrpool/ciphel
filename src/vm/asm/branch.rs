use super::operation::{OpPrimitive, PopNum};

use crate::vm::{
    allocator::{heap::Heap, stack::Stack},
    asm::operation::GetNumFrom,
    program::Program,
    runtime::RuntimeError,
    scheduler::Executable,
    stdio::StdIO,
};

use num_traits::ToBytes;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Label {
    pub id: Ulid,
    pub name: String,
}

impl Label {
    pub fn gen() -> Ulid {
        Ulid::new()
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Label {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm_label(engine, &self.name);
    }
}
impl crate::vm::AsmWeight for Label {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Label {
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
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Call {
    From { label: Ulid, param_size: usize },
    // Stack,
    Function { param_size: usize },
    Closure { param_size: usize },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Call {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            Call::From { label, param_size } => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_asm(engine, &format!("call {label} {param_size}"));
            }
            Call::Function { param_size } => stdio.push_asm(engine, "call"),
            Call::Closure { param_size } => stdio.push_asm(engine, "call"),
        }
    }
}
impl crate::vm::AsmWeight for Call {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Call {
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
        let (caller_data, param_size, function_offset) = match *self {
            Call::From { label, param_size } => {
                let Some(function_offset) = program.get_cursor_from_label(&label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                (None, param_size, function_offset)
            }
            Call::Function { param_size } => {
                let function_offset = OpPrimitive::pop_num::<u64>(stack)? as usize;
                (Some(function_offset as u64), param_size, function_offset)
            }
            Call::Closure { param_size } => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let function_offset =
                    OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                (Some(address.into(stack)), param_size, function_offset)
            }
        };

        let _ = stack.open_frame(param_size, scheduler.cursor.get() + 1, caller_data)?;

        scheduler.jump(function_offset);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Goto {
    pub label: Option<Ulid>,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Goto {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self.label {
            Some(label) => {
                let label = program
                    .get_label_name(&label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_asm(engine, &format!("goto {label}"));
            }
            None => stdio.push_asm(engine, "goto"),
        }
    }
}
impl crate::vm::AsmWeight for Goto {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for Goto {
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
        match self.label {
            Some(label) => {
                let Some(idx) = program.get_cursor_from_label(&label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                scheduler.jump(idx);
                Ok(())
            }
            None => {
                let idx = OpPrimitive::pop_num::<u64>(stack)? as usize;
                scheduler.jump(idx);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchIf {
    pub else_label: Ulid,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for BranchIf {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        let label = program
            .get_label_name(&self.else_label)
            .unwrap_or("".to_string().into())
            .to_string();
        stdio.push_asm(engine, &format!("else_goto {label}"));
    }
}

impl crate::vm::AsmWeight for BranchIf {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for BranchIf {
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
        let condition = OpPrimitive::pop_bool(stack)?;

        let Some(else_label) = program.get_cursor_from_label(&self.else_label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        scheduler.jump(if condition {
            scheduler.cursor.get() + 1
        } else {
            else_label
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum BranchTry {
    StartTry { else_label: Ulid },
    EndTry,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for BranchTry {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            BranchTry::StartTry { else_label } => {
                let label = program
                    .get_label_name(&else_label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_asm(engine, &format!("try_else {label}"));
            }
            BranchTry::EndTry => stdio.push_asm(engine, &format!("try_end")),
        }
    }
}
impl crate::vm::AsmWeight for BranchTry {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for BranchTry {
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
            BranchTry::StartTry { else_label } => {
                scheduler.push_catch(else_label);
            }
            BranchTry::EndTry => {
                scheduler.pop_catch();
            }
        }
        scheduler.next();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    pub size: usize,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for Return {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, &format!("return {0}", self.size))
    }
}
impl crate::vm::AsmWeight for Return {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::MEDIUM
    }
}

#[derive(Debug, Clone)]
pub struct CloseFrame;

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for CloseFrame {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        stdio.push_asm(engine, "return")
    }
}
impl crate::vm::AsmWeight for CloseFrame {
    fn weight(&self) -> crate::vm::Weight {
        crate::vm::Weight::ZERO
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Return {
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
        let return_pointer = stack.return_pointer;
        let _ = stack.close_frame(self.size)?;
        scheduler.jump(return_pointer);
        Ok(())
    }
}
impl<E: crate::vm::external::Engine> Executable<E> for CloseFrame {
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
        todo!()
    }
}
