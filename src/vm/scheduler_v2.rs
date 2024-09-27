use std::{io::Cursor, marker::PhantomData, ops::ControlFlow};

use ulid::Ulid;

use super::{
    error_handler::ErrorHandler,
    program::{Instruction, Program},
    runtime::RuntimeError,
    AsmName, AsmWeight, Weight,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext {}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {}
    }
}

pub trait Executable<E: crate::vm::external::Engine> {
    fn execute<P: crate::vm::scheduler_v2::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler_v2::Scheduler<P>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler_v2::ExecutionContext,
    ) -> Result<(), super::runtime::RuntimeError>;
}

pub trait SchedulingPolicy: Default {
    fn accept<E: crate::vm::external::Engine>(&self, weight: Weight, engine: &E) -> bool;
    fn defer<E: crate::vm::external::Engine>(&self, energy: usize, engine: &mut E);
    fn schedule<'a, E: crate::vm::external::Engine>(
        input: impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>,
    ) -> impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>
    where
        Self: 'a,
        <E as super::external::ExternThreadHandler>::TID: 'a;
}

pub struct ToCompletion;

impl Default for ToCompletion {
    fn default() -> Self {
        Self {}
    }
}

impl SchedulingPolicy for ToCompletion {
    fn accept<E: crate::vm::external::Engine>(&self, weight: Weight, engine: &E) -> bool {
        true
    }

    fn defer<E: crate::vm::external::Engine>(&self, energy: usize, engine: &mut E) {}

    fn schedule<'a, E: crate::vm::external::Engine>(
        input: impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>,
    ) -> impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>
    where
        Self: 'a,
        <E as super::external::ExternThreadHandler>::TID: 'a,
    {
        input
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ProgramCursor {
    Idle(usize),
    Running(usize),
}

impl Default for ProgramCursor {
    fn default() -> Self {
        Self::Running(0)
    }
}

impl ProgramCursor {
    pub fn get(&self) -> usize {
        match *self {
            ProgramCursor::Idle(cursor) => cursor,
            ProgramCursor::Running(cursor) => cursor,
        }
    }
    pub fn update<E: crate::vm::external::Engine>(&mut self, program: &Program<E>) {
        let cursor = self.get();
        if program.instructions.get(cursor).is_none() {
            *self = ProgramCursor::Idle(cursor);
        }
    }
}

pub struct Scheduler<P: SchedulingPolicy> {
    pub cursor: ProgramCursor,
    error_handler: ErrorHandler,
    policy: P,
}

impl<P: SchedulingPolicy> Default for Scheduler<P> {
    fn default() -> Self {
        Self {
            cursor: ProgramCursor::default(),
            error_handler: ErrorHandler::default(),
            policy: P::default(),
        }
    }
}

impl<P: SchedulingPolicy> Scheduler<P> {
    pub fn next(&mut self) {
        match self.cursor {
            ProgramCursor::Idle(cursor) => self.cursor = ProgramCursor::Running(cursor + 1),
            ProgramCursor::Running(cursor) => self.cursor = ProgramCursor::Running(cursor + 1),
        }
    }
    pub fn jump(&mut self, to: usize) {
        match self.cursor {
            ProgramCursor::Idle(_) => self.cursor = ProgramCursor::Running(to),
            ProgramCursor::Running(_) => self.cursor = ProgramCursor::Running(to),
        }
    }
    pub fn push_catch(&mut self, label: &Ulid) {
        self.error_handler.push_catch(label);
    }

    pub fn pop_catch(&mut self) {
        self.error_handler.pop_catch();
    }

    pub fn prepare(&mut self) {}

    fn select<'a, E: crate::vm::external::Engine>(
        &self,
        program: &'a Program<E>,
    ) -> Result<Option<&'a Instruction<E>>, RuntimeError> {
        let ProgramCursor::Running(cursor) = self.cursor else {
            return Ok(None);
        };
        let Some(instruction) = program.instructions.get(cursor) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        Ok(Some(instruction))
    }

    pub fn run<E: crate::vm::external::Engine>(
        &mut self,
        program: &Program<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        // context: &crate::vm::scheduler_v2::ExecutionContext,
    ) -> Result<ControlFlow<(), ()>, RuntimeError> {
        let Some(instruction) = self.select(program)? else {
            return Ok(ControlFlow::Break(()));
        };

        let weight = instruction.weight();
        if self.policy.accept::<E>(weight, engine) {
            self.policy.defer(weight.get(), engine);

            instruction.name(stdio, program, engine);
            match instruction.execute(
                program,
                self,
                stack,
                heap,
                stdio,
                engine,
                &crate::vm::scheduler_v2::ExecutionContext::default(),
            ) {
                Ok(_) => {}
                Err(error) => self.jump(self.error_handler.catch(error, program)?),
            }
            self.cursor.update(program);

            Ok(ControlFlow::Continue(()))
        } else {
            Ok(ControlFlow::Break(()))
        }
    }
}
