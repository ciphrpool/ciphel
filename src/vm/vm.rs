use std::{cell::RefCell, rc::Rc};

use crate::semantic::scope::ScopeApi;

use super::{
    allocator::{stack::StackError, Memory},
    strips::Strip,
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
    Default,
}

pub trait GenerateCode<Scope: ScopeApi> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Executable {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError>;
}

pub trait DeserializeFrom<Scope: ScopeApi> {
    type Output;
    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError>;
}
