use super::{allocator::Memory, strips::Strip};

#[derive(Debug, Clone)]
pub enum CodeGenerationError {
    Default,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UnsupportedOperation,
    MathError,
    Default,
}

pub trait GenerateCode {
    fn gencode(&self) -> Result<Vec<Strip>, CodeGenerationError>;
}

pub trait Executable {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError>;
}
