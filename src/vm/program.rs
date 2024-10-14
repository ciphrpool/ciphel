use std::collections::HashMap;

use ulid::Ulid;

use super::{
    asm::{branch::Label, Asm},
    external::Engine,
    scheduler::Executable,
    AsmName, AsmWeight,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    OPEN,
    CLOSE,
    COMMITED,
    REVERTED,
}

pub enum Instruction<E: Engine> {
    Extern(E::Function),
    Asm(Asm),
}

impl<E: Engine> AsmWeight for Instruction<E> {
    fn weight(&self) -> super::Weight {
        match self {
            Instruction::Extern(value) => value.weight(),
            Instruction::Asm(value) => value.weight(),
        }
    }
}

impl<E: Engine> AsmName<E> for Instruction<E> {
    fn name(&self, stdio: &mut super::stdio::StdIO, program: &self::Program<E>, engine: &mut E) {
        match self {
            Instruction::Extern(value) => value.name(stdio, program, engine),
            Instruction::Asm(value) => value.name(stdio, program, engine),
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for Instruction<E> {
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
    ) -> Result<(), super::runtime::RuntimeError> {
        match self {
            Instruction::Extern(value) => value.execute(
                program,
                scheduler,
                signal_handler,
                stack,
                heap,
                stdio,
                engine,
                context,
            ),
            Instruction::Asm(value) => value.execute(
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

pub struct Program<E: Engine> {
    pub instructions: Vec<Instruction<E>>,
    pub labels: HashMap<Ulid, (usize, Box<str>)>,
}

impl<E: Engine> Default for Program<E> {
    fn default() -> Self {
        Self {
            instructions: Default::default(),
            labels: Default::default(),
        }
    }
}

impl<E: Engine> Program<E> {
    pub fn push(&mut self, value: Asm) {
        self.instructions.push(Instruction::Asm(value));
    }
    pub fn push_extern(&mut self, value: E::Function) {
        self.instructions.push(Instruction::Extern(value));
    }
    pub fn push_label(&mut self, label: String) -> Ulid {
        let id = Ulid::new();

        self.labels
            .insert(id, (self.instructions.len(), label.clone().into()));
        self.instructions
            .push(Instruction::Asm(Asm::Label(Label { id, name: label })));
        id
    }
    pub fn push_label_by_id(&mut self, id: Ulid, label: String) -> Ulid {
        self.labels
            .insert(id, (self.instructions.len(), label.clone().into()));
        self.instructions
            .push(Instruction::Asm(Asm::Label(Label { id, name: label })));
        id
    }
    pub fn get_cursor_from_label(&self, label: &Ulid) -> Option<usize> {
        self.labels.get(label).map(|(i, _)| i).cloned()
    }

    pub fn get_label_name(&self, label: &Ulid) -> Option<Box<str>> {
        self.labels.get(label).map(|(_, name)| name).cloned()
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Instruction<E>>,
    {
        self.instructions.extend(iter);
    }

    pub fn merge(&mut self, other: Program<E>) {
        self.instructions.extend(other.instructions);
        self.labels.extend(other.labels);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}
