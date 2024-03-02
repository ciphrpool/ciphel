use crate::semantic::MutRc;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use ulid::Ulid;

use self::branch::Label;

use super::{
    allocator::Memory,
    platform,
    scheduler::Thread,
    vm::{self, Executable, Runtime, RuntimeError},
};
pub mod alloc;
pub mod branch;
pub mod locate;
mod math_operation;
pub mod memcopy;
pub mod operation;
pub mod serialize;

#[derive(Debug, Clone)]
pub struct CasmProgram {
    pub main: MutRc<Vec<Casm>>,
    pub cursor: Cell<usize>,
    pub labels: MutRc<HashMap<Ulid, usize>>,
}

impl Default for CasmProgram {
    fn default() -> Self {
        Self {
            main: Default::default(),
            cursor: Cell::new(0),
            labels: Default::default(),
        }
    }
}

impl CasmProgram {
    pub fn push(&self, value: Casm) {
        let mut borrowed = self.main.as_ref().borrow_mut();
        borrowed.push(value);
    }

    pub fn incr(&self) {
        self.cursor.set(self.cursor.get() + 1)
    }

    pub fn push_label(&self, label: String) -> Ulid {
        let id = Ulid::new();
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_labels.insert(id, borrowed_main.len());
        borrowed_main.push(Casm::Label(Label { id, name: label }));
        id
    }
    pub fn push_label_id(&self, id: Ulid, label: String) -> Ulid {
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_labels.insert(id, borrowed_main.len());
        borrowed_main.push(Casm::Label(Label { id, name: label }));
        id
    }

    pub fn get(&self, label: &Ulid) -> Option<usize> {
        let borrowed_labels = self.labels.as_ref().borrow();
        borrowed_labels.get(label).cloned()
    }

    pub fn extend<I>(&self, iter: I)
    where
        I: IntoIterator<Item = Casm>,
    {
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_main.extend(iter);
    }

    pub fn merge(&self, other: CasmProgram) {
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_main.extend(other.main.as_ref().take());
        borrowed_labels.extend(other.labels.as_ref().take());
    }

    pub fn len(&self) -> usize {
        let borrowed_main = self.main.as_ref().borrow();
        borrowed_main.len()
    }
    pub fn execute<'runtime>(&self, thread: &Thread) -> Result<(), vm::RuntimeError> {
        let borrowed_main = self.main.as_ref().borrow();
        loop {
            let cursor = self.cursor.get();
            match borrowed_main.get(cursor) {
                Some(instruction) => {
                    // dbg!((cursor, instruction, thread.env.stack.top()));

                    match instruction.execute(thread) {
                        Ok(_) => {}
                        Err(RuntimeError::Exit) => return Ok(()),
                        Err(e) => {
                            todo!("{:?} in {:?}", e, instruction)
                        }
                    }
                }
                None => {
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Casm {
    Platform(platform::LibCasm),
    StackFrame(alloc::StackFrame),
    Alloc(alloc::Alloc),
    MemCopy(memcopy::MemCopy),
    Operation(operation::Operation),
    Serialize(serialize::Serialized),
    Access(alloc::Access),
    Locate(locate::Locate),
    If(branch::BranchIf),
    Assign(alloc::Assign),
    Label(branch::Label),
    Call(branch::Call),
    Goto(branch::Goto),
    Switch(branch::BranchTable),
}

impl Executable for Casm {
    fn execute(&self, thread: &Thread) -> Result<(), vm::RuntimeError> {
        match self {
            Casm::Operation(value) => value.execute(thread),
            Casm::StackFrame(value) => value.execute(thread),
            Casm::Serialize(value) => value.execute(thread),
            Casm::Access(value) => value.execute(thread),
            Casm::If(value) => value.execute(thread),
            Casm::Assign(value) => value.execute(thread),
            Casm::Label(value) => value.execute(thread),
            Casm::Call(value) => value.execute(thread),
            Casm::Goto(value) => value.execute(thread),
            Casm::Alloc(value) => value.execute(thread),
            Casm::MemCopy(value) => value.execute(thread),
            Casm::Switch(value) => value.execute(thread),
            Casm::Locate(value) => value.execute(thread),
            Casm::Platform(value) => value.execute(thread),
        }
    }
}
