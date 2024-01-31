use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::semantic::scope::ScopeApi;

use super::{allocator::Memory, strips::Strip};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    UnresolvedError,
    Default,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UnsupportedOperation,
    MathError,
    Default,
}

pub trait GenerateCode<Scope: ScopeApi> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        codes: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError>;
}

pub trait Executable {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError>;
}
