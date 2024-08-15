use std::collections::HashMap;
use ulid::Ulid;

use crate::ast::statements::{Statement, WithLine};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    OPEN,
    CLOSE,
    COMMITED,
    REVERTED,
}

#[derive(Debug, Clone)]
pub struct CasmProgram {
    pub main: Vec<Casm>,
    pub statements_buffer: Vec<WithLine<Statement>>,
    pub in_transaction: TransactionState,
    cursor: usize,
    pub labels: HashMap<Ulid, (usize, Box<str>)>,
    pub catch_stack: Vec<Ulid>,
}

impl Default for CasmProgram {
    fn default() -> Self {
        Self {
            main: Default::default(),
            cursor: Default::default(),
            labels: Default::default(),
            catch_stack: Default::default(),
            statements_buffer: Vec::default(),
            in_transaction: TransactionState::CLOSE,
        }
    }
}

impl CasmProgram {
    pub fn push(&mut self, value: Casm) {
        self.main.push(value);
    }
    pub fn cursor_is_at_end(&self) -> bool {
        self.cursor == self.main.len()
    }
    pub fn cursor_to_end(&mut self) {
        self.cursor = self.main.len()
    }
    pub fn incr(&mut self) {
        self.cursor += 1;
    }
    pub fn cursor_set(&mut self, offset: usize) {
        self.cursor = offset;
    }
    pub fn cursor_get(&self) -> usize {
        self.cursor
    }
    pub fn push_label(&mut self, label: String) -> Ulid {
        let id = Ulid::new();

        self.labels
            .insert(id, (self.main.len(), label.clone().into()));
        self.main.push(Casm::Label(Label { id, name: label }));
        id
    }
    pub fn push_label_id(&mut self, id: Ulid, label: String) -> Ulid {
        self.labels
            .insert(id, (self.main.len(), label.clone().into()));
        self.main.push(Casm::Label(Label { id, name: label }));
        id
    }

    pub fn push_data(&mut self, data: Data) -> Ulid {
        let id = Ulid::new();
        self.labels
            .insert(id, (self.main.len(), "data".to_string().into()));
        self.main.push(Casm::Data(data));
        id
    }

    pub fn push_catch(&mut self, label: &Ulid) {
        self.catch_stack.push(*label);
    }

    pub fn pop_catch(&mut self) {
        self.catch_stack.pop();
    }

    pub fn catch(&mut self, err: RuntimeError) -> Result<(), RuntimeError> {
        if let Some(label) = self.catch_stack.last() {
            let Some(idx) = self.get(&label) else {
                return Err(RuntimeError::CodeSegmentation);
            };
            self.cursor_set(idx);
            Ok(())
        } else {
            Err(err)
        }
    }

    pub fn get(&self, label: &Ulid) -> Option<usize> {
        self.labels.get(label).map(|(i, _)| i).cloned()
    }

    pub fn get_label_name(&self, label: &Ulid) -> Option<Box<str>> {
        self.labels.get(label).map(|(_, name)| name).cloned()
    }

    pub fn data_at_offset(&self, offset: usize) -> Result<Vec<Box<[u8]>>, RuntimeError> {
        match self.main.get(offset) {
            Some(Casm::Data(data::Data::Dump { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
    }

    pub fn table_at_offset(&self, offset: usize) -> Result<Vec<Ulid>, RuntimeError> {
        match self.main.get(offset) {
            Some(Casm::Data(data::Data::Table { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Casm>,
    {
        self.main.extend(iter);
    }

    pub fn merge(&mut self, other: CasmProgram) {
        self.main.extend(other.main);
        self.labels.extend(other.labels);
    }

    pub fn len(&self) -> usize {
        self.main.len()
    }
    pub fn evaluate(
        &mut self,
        callback: impl FnOnce(&mut Self, &Casm) -> Result<(), vm::RuntimeError>,
    ) -> Result<(), vm::RuntimeError> {
        let instruction = match self.main.get(self.cursor) {
            Some(instruction) => instruction.clone(), // Clone to avoid borrow conflict
            None => return Err(RuntimeError::CodeSegmentation),
        };
        callback(self, &instruction)
    }
    pub fn execute<'runtime, G: crate::GameEngineStaticFn + Clone>(
        &mut self,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), vm::RuntimeError> {
        loop {
            let cursor = self.cursor; // Save cursor to avoid borrowing self
            let instruction = match self.main.get(cursor) {
                Some(instruction) => instruction.clone(), // Clone to avoid borrow conflict
                None => return Ok(()),
            };
            instruction.name(stdio, self, engine);
            match instruction.execute(self, stack, heap, stdio, engine, tid) {
                Ok(_) => {}
                Err(RuntimeError::Signal(Signal::EXIT)) => return Ok(()),
                Err(RuntimeError::AssertError) => return Ok(()),
                Err(e) => match self.catch(e) {
                    Ok(_) => {}
                    Err(e) => panic!("{:?} in {:?}", e, instruction),
                },
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
    AccessIdx(alloc::CheckIndex),
    Locate(locate::Locate),
    If(branch::BranchIf),
    Try(branch::BranchTry),
    // Assign(alloc::Assign),
    LocateUTF8Char(locate::LocateUTF8Char),
    Label(branch::Label),
    Call(branch::Call),
    Goto(branch::Goto),
    Switch(branch::BranchTable),
    Pop(usize),
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Casm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            Casm::Operation(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::StackFrame(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Data(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Access(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::AccessIdx(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::If(value) => value.execute(program, stack, heap, stdio, engine, tid),
            // Casm::Assign(value) => value.execute(program, stack, heap, stdio,engine),
            Casm::Label(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Call(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Goto(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Alloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Mem(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Switch(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Locate(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::LocateUTF8Char(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Platform(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Pop(size) => {
                stack.pop(*size)?;
                program.incr();
                Ok(())
            }
            Casm::Realloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Free(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Casm::Try(value) => value.execute(program, stack, heap, stdio, engine, tid),
        }
    }
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Casm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Casm::Platform(value) => value.name(stdio, program, engine),
            Casm::StackFrame(value) => value.name(stdio, program, engine),
            Casm::Alloc(value) => value.name(stdio, program, engine),
            Casm::Realloc(value) => value.name(stdio, program, engine),
            Casm::Free(value) => value.name(stdio, program, engine),
            Casm::Mem(value) => value.name(stdio, program, engine),
            Casm::Operation(value) => value.name(stdio, program, engine),
            Casm::Data(value) => value.name(stdio, program, engine),
            Casm::Access(value) => value.name(stdio, program, engine),
            Casm::AccessIdx(value) => value.name(stdio, program, engine),
            Casm::Locate(value) => value.name(stdio, program, engine),
            Casm::If(value) => value.name(stdio, program, engine),
            // Casm::Assign(value) => value.name(stdio, program,engine),
            Casm::LocateUTF8Char(value) => value.name(stdio, program, engine),
            Casm::Label(value) => value.name(stdio, program, engine),
            Casm::Call(value) => value.name(stdio, program, engine),
            Casm::Goto(value) => value.name(stdio, program, engine),
            Casm::Switch(value) => value.name(stdio, program, engine),
            Casm::Try(value) => value.name(stdio, program, engine),
            Casm::Pop(n) => stdio.push_casm(engine, &format!("pop {n}")),
        }
    }
}
