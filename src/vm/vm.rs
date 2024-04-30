use ulid::Ulid;

use crate::semantic::scope::scope::Scope;
use crate::semantic::MutRc;

use super::{
    allocator::{
        heap::{Heap, HeapError},
        stack::StackError,
        vtable::VTableError,
    },
    casm::CasmProgram,
    scheduler::{Env, Thread},
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
    Default,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub heap: Heap,
    pub stdio: StdIO,
    pub threads: Vec<Env>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            heap: Heap::new(),
            stdio: StdIO::default(),
            threads: Default::default(),
        }
    }

    pub fn spawn(&mut self) -> Result<usize, RuntimeError> {
        let env = Env::default();
        let tid = self.threads.len();
        self.threads.push(env);
        Ok(tid)
    }

    pub fn get<'runtime>(&'runtime self, tid: usize) -> Result<Thread<'runtime>, RuntimeError> {
        self.threads
            .get(tid)
            .ok_or(RuntimeError::Default)
            .map(|env| Thread {
                env,
                tid,
                runtime: self,
            })
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError>;
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
