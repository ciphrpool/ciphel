use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::semantic::{scope::ScopeApi, MutRc};

use super::{
    allocator::{stack::StackError, Memory},
    casm::{Casm, CasmProgram},
};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    UnresolvedError,
    Default,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackError(StackError),
    Deserialization,
    UnsupportedOperation,
    MathError,
    Exit,
    CodeSegmentation,
    Default,
}

pub trait GenerateCode<Scope: ScopeApi> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Executable {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError>;
}

pub trait DeserializeFrom<Scope: ScopeApi> {
    type Output;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError>;
}
