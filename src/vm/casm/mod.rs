use crate::semantic::MutRc;
use std::{cell::Cell, collections::HashMap, io, slice::Iter};
use ulid::Ulid;

use self::{branch::Label, data::Data};

use super::{
    allocator::{heap::Heap, stack::Stack},
    platform,
    stdio::StdIO,
    vm::{self, CasmMetadata, Executable, RuntimeError, Signal},
};
pub mod alloc;
pub mod branch;
pub mod data;
pub mod locate;
mod math_operation;
pub mod mem;
pub mod operation;

#[derive(Debug, Clone)]
pub struct CasmProgram {
    pub main: MutRc<Vec<Casm>>,
    cursor: Cell<usize>,
    pub labels: MutRc<HashMap<Ulid, (usize, Box<str>)>>,
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
    pub fn cursor_is_at_end(&self) -> bool {
        self.cursor.get() == self.main.as_ref().borrow().len()
    }
    pub fn incr(&self) {
        self.cursor.set(self.cursor.get() + 1)
    }
    pub fn cursor_set(&self, offset: usize) {
        self.cursor.set(offset);
    }
    pub fn cursor_get(&self) -> usize {
        self.cursor.get()
    }
    pub fn push_label(&self, label: String) -> Ulid {
        let id = Ulid::new();
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_labels.insert(id, (borrowed_main.len(), label.clone().into()));
        borrowed_main.push(Casm::Label(Label { id, name: label }));
        id
    }
    pub fn push_label_id(&self, id: Ulid, label: String) -> Ulid {
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_labels.insert(id, (borrowed_main.len(), label.clone().into()));
        borrowed_main.push(Casm::Label(Label { id, name: label }));
        id
    }

    pub fn push_data(&self, data: Data) -> Ulid {
        let id = Ulid::new();
        let mut borrowed_labels = self.labels.as_ref().borrow_mut();
        let mut borrowed_main = self.main.as_ref().borrow_mut();
        borrowed_labels.insert(id, (borrowed_main.len(), "data".into()));
        borrowed_main.push(Casm::Data(data));
        id
    }

    pub fn get(&self, label: &Ulid) -> Option<usize> {
        let borrowed_labels = self.labels.as_ref().borrow();
        borrowed_labels.get(label).map(|(i, _)| i).cloned()
    }

    pub fn get_label_name(&self, label: &Ulid) -> Option<Box<str>> {
        let borrowed_labels = self.labels.as_ref().borrow();
        borrowed_labels.get(label).map(|(_, name)| name).cloned()
    }

    pub fn data_at_offset(&self, offset: usize) -> Result<Vec<Box<[u8]>>, RuntimeError> {
        let borrowed_main = self.main.as_ref().borrow();
        match borrowed_main.get(offset) {
            Some(Casm::Data(data::Data::Dump { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
    }

    pub fn table_at_offset(&self, offset: usize) -> Result<Vec<Ulid>, RuntimeError> {
        let borrowed_main = self.main.as_ref().borrow();
        match borrowed_main.get(offset) {
            Some(Casm::Data(data::Data::Table { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
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
    pub fn evaluate(
        &self,
        callback: impl FnOnce(&Casm) -> Result<(), vm::RuntimeError>,
    ) -> Result<(), vm::RuntimeError> {
        let borrowed_main = self.main.as_ref().borrow();
        let cursor = self.cursor.get();
        match borrowed_main.get(cursor) {
            Some(instruction) => callback(instruction),
            None => {
                return Err(RuntimeError::CodeSegmentation);
            }
        }
    }
    pub fn execute<'runtime>(
        &self,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), vm::RuntimeError> {
        let borrowed_main = self.main.as_ref().borrow();
        loop {
            let cursor = self.cursor.get();
            match borrowed_main.get(cursor) {
                Some(instruction) => {
                    // dbg!((cursor, instruction, stack.top(),));
                    // let mut buffer = String::new();
                    // io::stdin().read_line(&mut buffer);
                    // instruction.name(stdio, self);
                    match instruction.execute(self, stack, heap, stdio) {
                        Ok(_) => {}
                        Err(RuntimeError::Signal(Signal::EXIT)) => return Ok(()),
                        Err(RuntimeError::AssertError) => return Ok(()),
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
    Realloc(alloc::Realloc),
    Free(alloc::Free),
    Mem(mem::Mem),
    Operation(operation::Operation),
    Data(data::Data),
    Access(alloc::Access),
    Locate(locate::Locate),
    If(branch::BranchIf),
    // Assign(alloc::Assign),
    LocateNextUTF8Char(locate::LocateNextUTF8Char),
    Label(branch::Label),
    Call(branch::Call),
    Goto(branch::Goto),
    Switch(branch::BranchTable),
    Pop(usize),
}

impl Executable for Casm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            Casm::Operation(value) => value.execute(program, stack, heap, stdio),
            Casm::StackFrame(value) => value.execute(program, stack, heap, stdio),
            Casm::Data(value) => value.execute(program, stack, heap, stdio),
            Casm::Access(value) => value.execute(program, stack, heap, stdio),
            Casm::If(value) => value.execute(program, stack, heap, stdio),
            // Casm::Assign(value) => value.execute(program, stack, heap, stdio),
            Casm::Label(value) => value.execute(program, stack, heap, stdio),
            Casm::Call(value) => value.execute(program, stack, heap, stdio),
            Casm::Goto(value) => value.execute(program, stack, heap, stdio),
            Casm::Alloc(value) => value.execute(program, stack, heap, stdio),
            Casm::Mem(value) => value.execute(program, stack, heap, stdio),
            Casm::Switch(value) => value.execute(program, stack, heap, stdio),
            Casm::Locate(value) => value.execute(program, stack, heap, stdio),
            Casm::LocateNextUTF8Char(value) => value.execute(program, stack, heap, stdio),
            Casm::Platform(value) => value.execute(program, stack, heap, stdio),
            Casm::Pop(size) => {
                stack.pop(*size).map_err(|e| e.into())?;
                program.incr();
                Ok(())
            }
            Casm::Realloc(value) => value.execute(program, stack, heap, stdio),
            Casm::Free(value) => value.execute(program, stack, heap, stdio),
        }
    }
}

impl CasmMetadata for Casm {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self {
            Casm::Platform(value) => value.name(stdio, program),
            Casm::StackFrame(value) => value.name(stdio, program),
            Casm::Alloc(value) => value.name(stdio, program),
            Casm::Realloc(value) => value.name(stdio, program),
            Casm::Free(value) => value.name(stdio, program),
            Casm::Mem(value) => value.name(stdio, program),
            Casm::Operation(value) => value.name(stdio, program),
            Casm::Data(value) => value.name(stdio, program),
            Casm::Access(value) => value.name(stdio, program),
            Casm::Locate(value) => value.name(stdio, program),
            Casm::If(value) => value.name(stdio, program),
            // Casm::Assign(value) => value.name(stdio, program),
            Casm::LocateNextUTF8Char(value) => value.name(stdio, program),
            Casm::Label(value) => value.name(stdio, program),
            Casm::Call(value) => value.name(stdio, program),
            Casm::Goto(value) => value.name(stdio, program),
            Casm::Switch(value) => value.name(stdio, program),
            Casm::Pop(n) => stdio.push_casm(&format!("pop {n}")),
        }
    }
}
