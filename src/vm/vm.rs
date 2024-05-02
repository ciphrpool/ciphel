use std::cell::Cell;

use ulid::Ulid;

use crate::semantic::scope::scope::Scope;
use crate::semantic::MutRc;

use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::{Stack, StackError},
        vtable::VTableError,
    },
    casm::CasmProgram,
    stdio::StdIO,
};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    UnresolvedError,
    Default,
}

pub type Tid = usize;

#[derive(Debug, Clone)]
pub enum Signal {
    SPAWN,
    CLOSE(Tid),
    WAIT(Tid),
    WAKE(Tid),
    SLEEP(usize),
    JOIN(Tid),
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackError(StackError),
    HeapError(HeapError),
    VTableError(VTableError),
    Deserialization,
    UnsupportedOperation,
    MathError,
    ReturnFlagError,
    InvalidUTF8Char,
    Exit,
    CodeSegmentation,
    IncorrectVariant,
    InvalidTID(usize),
    TooManyThread,
    Signal(Signal),
    Default,
}

pub const MAX_THREAD_COUNT: usize = 4;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub tid_count: Cell<usize>,
    pub threads: Vec<(MutRc<Scope>, Stack, CasmProgram, Tid)>,
}

impl Runtime {
    pub fn new() -> (Self, Heap, StdIO) {
        (
            Self {
                tid_count: Cell::new(0),
                threads: Vec::with_capacity(MAX_THREAD_COUNT),
            },
            Heap::new(),
            StdIO::default(),
        )
    }

    pub fn spawn(&mut self) -> Result<Tid, RuntimeError> {
        if self.threads.len() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }
        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.tid_count.get() + 1;
        self.tid_count.set(tid);
        self.threads.push((scope, stack, program, tid));
        Ok(tid)
    }

    pub fn spawn_with_tid(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        if self.threads.len() >= MAX_THREAD_COUNT {
            return Err(RuntimeError::TooManyThread);
        }
        let scope = Scope::new();
        let program = CasmProgram::default();
        let stack = Stack::new();
        self.tid_count.set(tid + 1);
        self.threads.push((scope, stack, program, tid));
        Ok(())
    }
    pub fn close(&mut self, tid: Tid) -> Result<(), RuntimeError> {
        let idx = self.threads.iter().position(|(_, _, _, id)| *id == tid);
        if let Some(idx) = idx {
            let _ = self.threads.remove(idx);
            Ok(())
        } else {
            Err(RuntimeError::InvalidTID(tid))
        }
    }

    pub fn spawn_with_scope(&mut self, scope: MutRc<Scope>) -> Result<usize, RuntimeError> {
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.tid_count.get() + 1;
        self.tid_count.set(tid);
        self.threads.push((scope, stack, program, tid));
        Ok(tid)
    }

    pub fn get_mut<'runtime>(
        &'runtime mut self,
        tid: Tid,
    ) -> Result<
        (
            &'runtime mut MutRc<Scope>,
            &'runtime mut Stack,
            &mut CasmProgram,
        ),
        RuntimeError,
    > {
        self.threads
            .iter_mut()
            .find(|(_, _, _, id)| *id == tid)
            .ok_or(RuntimeError::InvalidTID(tid))
            .map(|(scope, stack, program, _)| (scope, stack, program))
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, (MutRc<Scope>, Stack, CasmProgram, Tid)> {
        self.threads.iter_mut()
    }
}

pub trait GenerateCode {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Executable {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError>;
}

pub trait CasmMetadata {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram);
    fn weight(&self) -> usize {
        1
    }
}

pub trait DeserializeFrom {
    type Output;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError>;
}

pub trait Printer {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;
}

pub trait NextItem {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;

    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: Ulid,
    ) -> Result<(), CodeGenerationError>;

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError>;
}
