use super::operation::OpPrimitive;
use super::CasmProgram;
use crate::vm::{
    allocator::Memory,
    vm::{Executable, RuntimeError},
};
use std::cell::Cell;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Label {
    pub id: Ulid,
    pub name: String,
}

impl Label {
    pub fn gen() -> Ulid {
        Ulid::new()
    }
}

impl Executable for Label {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        program.cursor.set(program.cursor.get() + 1);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub label: Ulid,
}

impl Executable for Call {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let Some(idx) = program.labels.get(&self.label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        program.cursor.set(*idx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Goto {
    pub label: Ulid,
}

impl Executable for Goto {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let Some(idx) = program.labels.get(&self.label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        program.cursor.set(*idx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BranchIf {
    pub else_label: Ulid,
}

impl Executable for BranchIf {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let condition = OpPrimitive::get_bool(memory)?;
        let Some(else_label) = program.labels.get(&self.else_label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        program.cursor.set(if condition {
            program.cursor.get() + 1
        } else {
            *else_label
        });
        Ok(())
        // if condition {
        //     self.then_stack.execute(instructions, memory)
        // } else {
        //     self.else_stack.execute(instructions, memory)
        // }
    }
}
