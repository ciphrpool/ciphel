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
    vm::{self, Executable, RuntimeError},
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
    pub main: Vec<Casm>,
    pub cursor: Cell<usize>,
    pub labels: HashMap<Ulid, usize>,
}

impl Default for CasmProgram {
    fn default() -> Self {
        Self {
            main: Vec::default(),
            cursor: Cell::new(0),
            labels: HashMap::default(),
        }
    }
}

impl CasmProgram {
    pub fn push(&mut self, value: Casm) {
        self.main.push(value);
    }

    pub fn push_label(&mut self, label: String) -> Ulid {
        let id = Ulid::new();
        self.labels.insert(id, self.main.len());
        self.main.push(Casm::Label(Label { id, name: label }));
        id
    }
    pub fn push_label_id(&mut self, id: Ulid, label: String) -> Ulid {
        self.labels.insert(id, self.main.len());
        self.main.push(Casm::Label(Label { id, name: label }));
        id
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Casm>,
    {
        self.main.extend(iter);
    }

    pub fn len(&self) -> usize {
        self.main.len()
    }
    pub fn execute(&self, memory: &Memory) -> Result<(), vm::RuntimeError> {
        loop {
            let cursor = self.cursor.get();
            // dbg!(&cursor);
            match self.main.get(cursor) {
                Some(instruction) => {
                    // dbg!((cursor, instruction, memory.stack.top()));

                    match instruction.execute(&self, memory) {
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
}

impl Executable for Casm {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), vm::RuntimeError> {
        match self {
            Casm::Operation(value) => value.execute(program, memory),
            Casm::StackFrame(value) => value.execute(program, memory),
            Casm::Serialize(value) => value.execute(program, memory),
            Casm::Access(value) => value.execute(program, memory),
            Casm::If(value) => value.execute(program, memory),
            Casm::Assign(value) => value.execute(program, memory),
            Casm::Label(value) => value.execute(program, memory),
            Casm::Call(value) => value.execute(program, memory),
            Casm::Goto(value) => value.execute(program, memory),
            Casm::Alloc(value) => value.execute(program, memory),
            Casm::MemCopy(value) => value.execute(program, memory),
            Casm::Locate(value) => value.execute(program, memory),
        }
    }
}
