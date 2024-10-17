use std::{default, ops::ControlFlow};

use ulid::Ulid;

use crate::vm::asm::operation::{GetNumFrom, OpPrimitive};

use super::{
    allocator::MemoryAddress,
    error_handler::ErrorHandler,
    external::{ExternExecutionContext, ExternProcessIdentifier, ExternThreadIdentifier},
    program::{Instruction, Program},
    runtime::{RuntimeError, ThreadState},
    AsmName, AsmWeight, Weight,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionContext<
    C: ExternExecutionContext,
    PID: ExternProcessIdentifier,
    TID: ExternThreadIdentifier<PID>,
> {
    pub external: C,
    pub tid: TID,
    pub pid: PID,
}

impl<C: ExternExecutionContext, PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>
    Default for ExecutionContext<C, PID, TID>
{
    fn default() -> Self {
        Self {
            external: C::default(),
            tid: TID::default(),
            pid: PID::default(),
        }
    }
}

pub trait Executable<E: crate::vm::external::Engine> {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<(), super::runtime::RuntimeError>;
}

pub trait SchedulingPolicy: Default {
    fn weight_to_energy(&self, weight: Weight) -> usize;

    fn accept<E: crate::vm::external::Engine>(&self, energy: usize, engine: &E) -> bool;
    fn defer<E: crate::vm::external::Engine>(&mut self, energy: usize, pid: E::PID, engine: &mut E);

    fn init_maf<E: crate::vm::external::Engine>(
        &mut self,
        tid: &E::TID,
        state: &super::runtime::ThreadState<E::PID, E::TID>,
    );

    fn init_watchdog(&mut self);
    fn watchdog(&mut self) -> ControlFlow<(), ()>;
    fn event_watchdog(&mut self) -> ControlFlow<(), ()>;

    fn schedule<'a, E: crate::vm::external::Engine>(
        input: impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>,
    ) -> impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>
    where
        Self: 'a,
        <E as super::external::ExternThreadHandler>::TID: 'a;
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
    pub fn update<E: crate::vm::external::Engine>(
        &mut self,
        program: &Program<E>,
        state: &mut ThreadState<E::PID, E::TID>,
    ) {
        let cursor = self.get();
        if program.instructions.get(cursor).is_none() {
            *self = ProgramCursor::Idle(cursor);
            *state = ThreadState::IDLE;
        } else if let ProgramCursor::Idle(cursor) = self {
            *self = ProgramCursor::Running(*cursor);
            if ThreadState::IDLE == *state {
                *state = ThreadState::RUNNING;
            }
        } else if ThreadState::IDLE == *state {
            *state = ThreadState::RUNNING;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    Once,
    Repetable,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventExclusivity {
    PerTID,
    PerPID,
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EventState {
    #[default]
    IDLE,
    Triggered,
    Running,
    Completed,
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct EventConf {
    pub kind: EventKind,
    pub exclu: EventExclusivity,
}

pub struct Event<E: crate::vm::external::Engine> {
    pub tid: E::TID,
    pub trigger: u64,
    pub callback: MemoryAddress,
    pub conf: EventConf,
    pub state: EventState,
}

pub struct Scheduler<P: SchedulingPolicy> {
    pub cursor: ProgramCursor,
    error_handler: ErrorHandler,
    pub policy: P,
    in_event: bool,
    early_break: bool,
}

impl<P: SchedulingPolicy> Default for Scheduler<P> {
    fn default() -> Self {
        Self {
            cursor: ProgramCursor::default(),
            error_handler: ErrorHandler::default(),
            policy: P::default(),
            in_event: false,
            early_break: false,
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
    pub fn signal_return(&mut self) {
        if self.in_event {
            self.early_break = true;
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
        tid: E::TID,
        state: &mut ThreadState<E::PID, E::TID>,
        program: &Program<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        signal_handler: &mut super::signal::SignalHandler<E>,
        // context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::PID, E::TID>,
    ) -> Result<ControlFlow<bool, ()>, RuntimeError> {
        let pid = tid.pid();
        let Some(instruction) = self.select(program)? else {
            return Ok(ControlFlow::Break(false));
        };

        let weight = instruction.weight();
        let energy = self.policy.weight_to_energy(weight);
        if self.policy.accept::<E>(energy, engine) {
            self.policy.defer(energy, pid.clone(), engine);

            instruction.name(stdio, program, engine);
            self.early_break = false;
            match instruction.execute(
                program,
                self,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                &crate::vm::scheduler::ExecutionContext {
                    external: E::FunctionContext::default(),
                    tid,
                    pid,
                },
            ) {
                Ok(_) => {}
                Err(error) => self.jump(self.error_handler.catch(error, program)?),
            }
            if self.early_break {
                self.early_break = false;
                return Ok(ControlFlow::Break(true));
            }
            self.cursor.update(program, state);

            Ok(ControlFlow::Continue(()))
        } else {
            Ok(ControlFlow::Break(false))
        }
    }

    pub fn run_event<E: crate::vm::external::Engine>(
        &mut self,
        callback_address: MemoryAddress,
        tid: E::TID,
        state: &mut ThreadState<E::PID, E::TID>,
        program: &Program<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        signal_handler: &mut super::signal::SignalHandler<E>,
    ) -> Result<(), RuntimeError> {
        let pid = tid.pid();
        let saved_cursor = self.cursor.clone();
        let function_offset =
            OpPrimitive::get_num_from::<u64>(callback_address, stack, heap)? as usize;

        let _ = stack.open_frame(0, saved_cursor.get(), Some(callback_address.into(stack)))?;

        self.jump(function_offset);
        self.in_event = true;

        loop {
            match self.run(
                tid.clone(),
                state,
                program,
                stack,
                heap,
                stdio,
                engine,
                signal_handler,
            )? {
                std::ops::ControlFlow::Continue(_) => match self.policy.event_watchdog() {
                    std::ops::ControlFlow::Continue(_) => continue,
                    std::ops::ControlFlow::Break(_) => {
                        break;
                    }
                },
                std::ops::ControlFlow::Break(true) => {
                    break;
                }
                std::ops::ControlFlow::Break(_) => {
                    break;
                }
            }
        }

        self.cursor = saved_cursor;
        self.in_event = false;
        Ok(())
    }
}

pub struct ToCompletion;

impl Default for ToCompletion {
    fn default() -> Self {
        Self {}
    }
}

impl SchedulingPolicy for ToCompletion {
    fn weight_to_energy(&self, weight: Weight) -> usize {
        1
    }

    fn accept<E: crate::vm::external::Engine>(&self, energy: usize, engine: &E) -> bool {
        true
    }

    fn defer<E: crate::vm::external::Engine>(
        &mut self,
        energy: usize,
        pid: E::PID,
        engine: &mut E,
    ) {
    }

    fn init_watchdog(&mut self) {}

    fn watchdog(&mut self) -> ControlFlow<(), ()> {
        ControlFlow::Continue(())
    }

    fn schedule<'a, E: crate::vm::external::Engine>(
        input: impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>,
    ) -> impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>
    where
        Self: 'a,
        <E as super::external::ExternThreadHandler>::TID: 'a,
    {
        input
    }

    fn init_maf<E: crate::vm::external::Engine>(
        &mut self,
        tid: &E::TID,
        state: &super::runtime::ThreadState<E::PID, E::TID>,
    ) {
    }

    fn event_watchdog(&mut self) -> ControlFlow<(), ()> {
        ControlFlow::Continue(())
    }
}

pub struct QueuePolicy {
    balance: usize,
}

impl Default for QueuePolicy {
    fn default() -> Self {
        Self {
            balance: Self::MAX_BALANCE,
        }
    }
}
impl QueuePolicy {
    pub const MAX_BALANCE: usize = 64;
}

impl SchedulingPolicy for QueuePolicy {
    fn weight_to_energy(&self, weight: Weight) -> usize {
        match weight {
            Weight::ZERO => 0,
            Weight::MAX => Self::MAX_BALANCE,
            Weight::CUSTOM(w) => w,
            Weight::LOW => 1,
            Weight::MEDIUM => 2,
            Weight::HIGH => 4,
            Weight::EXTREME => 8,
            Weight::END => {
                if self.balance > 0 {
                    self.balance
                } else {
                    1
                }
            }
        }
    }

    fn accept<E: crate::vm::external::Engine>(&self, energy: usize, engine: &E) -> bool {
        self.balance.checked_sub(energy).is_some()
    }

    fn defer<E: crate::vm::external::Engine>(
        &mut self,
        energy: usize,
        pid: E::PID,
        engine: &mut E,
    ) {
        self.balance = self.balance.checked_sub(energy).unwrap_or(0);
    }

    fn init_watchdog(&mut self) {}

    fn watchdog(&mut self) -> ControlFlow<(), ()> {
        ControlFlow::Continue(())
    }

    fn schedule<'a, E: crate::vm::external::Engine>(
        input: impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>,
    ) -> impl Iterator<Item = (&'a E::TID, &'a mut super::runtime::Thread<Self>)>
    where
        Self: 'a,
        <E as super::external::ExternThreadHandler>::TID: 'a,
    {
        input
    }

    fn init_maf<E: crate::vm::external::Engine>(
        &mut self,
        tid: &E::TID,
        state: &super::runtime::ThreadState<E::PID, E::TID>,
    ) {
        self.balance = Self::MAX_BALANCE;
    }

    fn event_watchdog(&mut self) -> ControlFlow<(), ()> {
        ControlFlow::Continue(())
    }
}
