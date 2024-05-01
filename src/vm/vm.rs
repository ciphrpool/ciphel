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
    Default,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub threads: Vec<(Stack, CasmProgram, usize)>,
}

impl Runtime {
    pub fn new() -> (Self, Heap, StdIO) {
        (
            Self {
                threads: Default::default(),
            },
            Heap::new(),
            StdIO::default(),
        )
    }

    pub fn spawn(&mut self) -> Result<usize, RuntimeError> {
        let program = CasmProgram::default();
        let stack = Stack::new();
        let tid = self.threads.len();
        self.threads.push((stack, program, tid));
        Ok(tid)
    }

    pub fn get_mut<'runtime>(
        &'runtime mut self,
        tid: usize,
    ) -> Result<(&'runtime mut Stack, &mut CasmProgram), RuntimeError> {
        self.threads
            .get_mut(tid)
            .ok_or(RuntimeError::InvalidTID(tid))
            .map(|(stack, program, _)| (stack, program))
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
