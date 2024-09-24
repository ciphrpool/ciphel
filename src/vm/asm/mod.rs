use branch::BranchIf;
use std::collections::HashMap;
use ulid::Ulid;

use crate::ast::statements::{Statement, WithLine};

use self::{branch::Label, data::Data};

use super::{
    allocator::{heap::Heap, stack::Stack},
    core,
    stdio::StdIO,
    vm::{self, CasmMetadata, Executable, GameEngineStaticFn, RuntimeError, Signal},
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
pub struct Program {
    pub main: Vec<Asm>,
    pub statements_buffer: Vec<WithLine<Statement>>,
    pub in_transaction: TransactionState,
    cursor: usize,
    pub labels: HashMap<Ulid, (usize, Box<str>)>,
    pub catch_stack: Vec<Ulid>,
}

impl Default for Program {
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

impl Program {
    pub fn push(&mut self, value: Asm) {
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
        self.main.push(Asm::Label(Label { id, name: label }));
        id
    }
    pub fn push_label_id(&mut self, id: Ulid, label: String) -> Ulid {
        self.labels
            .insert(id, (self.main.len(), label.clone().into()));
        self.main.push(Asm::Label(Label { id, name: label }));
        id
    }

    pub fn push_data(&mut self, data: Data) -> Ulid {
        let id = Ulid::new();
        self.labels
            .insert(id, (self.main.len(), "data".to_string().into()));
        self.main.push(Asm::Data(data));
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
            Some(Asm::Data(data::Data::Dump { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
    }

    pub fn table_at_offset(&self, offset: usize) -> Result<Vec<Ulid>, RuntimeError> {
        match self.main.get(offset) {
            Some(Asm::Data(data::Data::Table { data })) => Ok(data.to_vec()),
            _ => Err(RuntimeError::CodeSegmentation),
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Asm>,
    {
        self.main.extend(iter);
    }

    pub fn merge(&mut self, other: Program) {
        self.main.extend(other.main);
        self.labels.extend(other.labels);
    }

    pub fn len(&self) -> usize {
        self.main.len()
    }

    pub fn current_instruction_weight<G: GameEngineStaticFn>(&self) -> usize {
        match self.main.get(self.cursor) {
            Some(instruction) => <Asm as CasmMetadata<G>>::weight(instruction).get(),
            None => 0,
        }
    }

    pub fn evaluate(
        &mut self,
        callback: impl FnOnce(&mut Self, &Asm) -> Result<(), vm::RuntimeError>,
    ) -> Result<(), vm::RuntimeError> {
        let instruction = match self.main.get(self.cursor) {
            Some(instruction) => instruction.clone(), // Clone to avoid borrow conflict
            None => return Err(RuntimeError::CodeSegmentation),
        };
        callback(self, &instruction)
    }
    pub fn execute<'runtime, G: crate::GameEngineStaticFn>(
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
            // std::io::stdin().read_line(&mut String::new());
            match instruction.execute(self, stack, heap, stdio, engine, tid) {
                Ok(_) => {}
                Err(RuntimeError::Signal(Signal::EXIT)) => return Ok(()),
                Err(RuntimeError::AssertError) => return Ok(()),
                Err(e) => match self.catch(e) {
                    Ok(_) => {}
                    Err(e) => panic!("{:?} in {:?}", e, instruction),
                },
            }
            // dbg!(stack.top());
        }
    }
}

#[derive(Debug, Clone)]
pub enum Asm {
    Core(core::CoreCasm),
    Return(branch::Return),
    Break(branch::Break),
    Continue(branch::Continue),
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

impl<G: crate::GameEngineStaticFn> Executable<G> for Asm {
    fn execute(
        &self,
        program: &mut Program,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            Asm::Operation(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Return(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Continue(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::CloseFrame(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Break(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Data(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Access(value) => value.execute(program, stack, heap, stdio, engine, tid),
            // Asm::AccessIdx(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::If(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Label(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Call(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Goto(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Alloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Mem(value) => value.execute(program, stack, heap, stdio, engine, tid),
            // Asm::Switch(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::OffsetIdx(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Offset(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::OffsetSP(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Locate(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Core(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Pop(size) => {
                stack.pop(*size)?;
                program.incr();
                Ok(())
            }
            Asm::Realloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Free(value) => value.execute(program, stack, heap, stdio, engine, tid),
            Asm::Try(value) => value.execute(program, stack, heap, stdio, engine, tid),
        }
    }
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Asm {
    fn name(&self, stdio: &mut StdIO, program: &mut Program, engine: &mut G) {
        match self {
            Asm::Core(value) => value.name(stdio, program, engine),
            Asm::Return(value) => value.name(stdio, program, engine),
            Asm::Continue(value) => value.name(stdio, program, engine),
            Asm::CloseFrame(value) => value.name(stdio, program, engine),
            Asm::Break(value) => value.name(stdio, program, engine),
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

    fn weight(&self) -> vm::CasmWeight {
        match self {
            Asm::Core(value) => <core::CoreCasm as CasmMetadata<G>>::weight(value),
            Asm::Return(value) => <branch::Return as CasmMetadata<G>>::weight(value),
            Asm::Continue(value) => <branch::Continue as CasmMetadata<G>>::weight(value),
            Asm::CloseFrame(value) => <branch::CloseFrame as CasmMetadata<G>>::weight(value),
            Asm::Break(value) => <branch::Break as CasmMetadata<G>>::weight(value),
            Asm::Alloc(value) => <alloc::Alloc as CasmMetadata<G>>::weight(value),
            Asm::Realloc(value) => <alloc::Realloc as CasmMetadata<G>>::weight(value),
            Asm::Free(value) => <alloc::Free as CasmMetadata<G>>::weight(value),
            Asm::Mem(value) => <mem::Mem as CasmMetadata<G>>::weight(value),
            Asm::Operation(value) => <operation::Operation as CasmMetadata<G>>::weight(value),
            Asm::Data(value) => <Data as CasmMetadata<G>>::weight(value),
            Asm::Access(value) => <alloc::Access as CasmMetadata<G>>::weight(value),
            // Asm::AccessIdx(value) => <CheckIndex as CasmMetadata<G>>::weight(value),
            Asm::OffsetSP(value) => {
                <locate::LocateOffsetFromStackPointer as CasmMetadata<G>>::weight(value)
            }
            Asm::Offset(value) => <locate::LocateOffset as CasmMetadata<G>>::weight(value),
            Asm::OffsetIdx(value) => <locate::LocateIndex as CasmMetadata<G>>::weight(value),
            Asm::Locate(value) => <locate::Locate as CasmMetadata<G>>::weight(value),
            Asm::If(value) => <BranchIf as CasmMetadata<G>>::weight(value),
            Asm::Try(value) => <branch::BranchTry as CasmMetadata<G>>::weight(value),
            Asm::Label(value) => <Label as CasmMetadata<G>>::weight(value),
            Asm::Call(value) => <branch::Call as CasmMetadata<G>>::weight(value),
            Asm::Goto(value) => <branch::Goto as CasmMetadata<G>>::weight(value),
            // Asm::Switch(value) => <BranchTable as CasmMetadata<G>>::weight(value),
            Asm::Pop(value) => vm::CasmWeight::LOW,
        }
    }
}
