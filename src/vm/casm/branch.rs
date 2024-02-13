use super::operation::OpPrimitive;
use super::CasmProgram;
use crate::vm::{
    allocator::{stack::Offset, Memory},
    vm::{Executable, RuntimeError},
};
use std::{cell::Cell, collections::HashMap};
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
    pub return_size: usize,
    pub param_size: usize,
}

impl Executable for Call {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let _ = memory
            .stack
            .frame(self.return_size, self.param_size, program.cursor.get() + 1)
            .map_err(|e| e.into())?;

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
    }
}

#[derive(Debug, Clone)]
pub struct BranchTable {
    pub table: HashMap<u64, Ulid>,
    pub else_label: Option<Ulid>,
}

impl Executable for BranchTable {
    fn execute(&self, program: &CasmProgram, memory: &Memory) -> Result<(), RuntimeError> {
        let variant = OpPrimitive::get_num8::<u64>(memory)?;
        match (self.table.get(&variant), self.else_label) {
            (None, None) => return Err(RuntimeError::IncorrectVariant),
            (None, Some(else_label)) => {
                let Some(idx) = program.labels.get(&else_label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                program.cursor.set(*idx);
                Ok(())
            }
            (Some(label), _) => {
                let Some(idx) = program.labels.get(label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                program.cursor.set(*idx);
                Ok(())
            }
        }
    }
}
