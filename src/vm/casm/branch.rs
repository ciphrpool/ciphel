use super::operation::OpPrimitive;
use super::CasmProgram;
use crate::{
    semantic::{scope::static_types::st_deserialize::extract_u64, AccessLevel},
    vm::{
        allocator::{stack::Offset, Memory},
        scheduler::Thread,
        vm::{Executable, RuntimeError},
    },
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
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        thread.env.program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Call {
    From {
        label: Ulid,
        return_size: usize,
        param_size: usize,
    },
    Stack,
}

impl Executable for Call {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let (return_size, param_size, function_offset) = match self {
            Call::From {
                label,
                return_size,
                param_size,
            } => {
                let Some(function_offset) = thread.env.program.get(&label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                (*return_size, *param_size, function_offset)
            }
            Call::Stack => {
                let return_size = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                let param_size = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                let function_offset = OpPrimitive::get_num8::<u64>(&thread.memory())? as usize;
                (return_size, param_size, function_offset)
            }
        };
        if param_size != 0 {
            let data = thread.env.stack.pop(param_size).map_err(|e| e.into())?;
            let _ = thread
                .env
                .stack
                .frame(return_size, param_size, thread.env.program.cursor.get() + 1)
                .map_err(|e| e.into())?;
            let _ = thread
                .env
                .stack
                .write(Offset::FP(0), AccessLevel::Direct, &data)
                .map_err(|e| e.into())?;
        } else {
            let _ = thread
                .env
                .stack
                .frame(return_size, param_size, thread.env.program.cursor.get() + 1)
                .map_err(|e| e.into())?;
        }

        thread.env.program.cursor_set(function_offset);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Goto {
    pub label: Ulid,
}

impl Executable for Goto {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let Some(idx) = thread.env.program.get(&self.label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        thread.env.program.cursor_set(idx);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BranchIf {
    pub else_label: Ulid,
}

impl Executable for BranchIf {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        let condition = OpPrimitive::get_bool(&thread.memory())?;
        let Some(else_label) = thread.env.program.get(&self.else_label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        thread.env.program.cursor_set(if condition {
            thread.env.program.cursor.get() + 1
        } else {
            else_label
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum BranchTableExprInfo {
    Primitive(usize),
    String,
}

#[derive(Debug, Clone)]
pub enum BranchTable {
    Swith {
        info: BranchTableExprInfo,
        table: Vec<(Vec<u8>, Ulid)>,
        else_label: Option<Ulid>,
    },
    Table {
        table: Vec<(u64, Ulid)>,
        else_label: Option<Ulid>,
    },
}

impl Executable for BranchTable {
    fn execute(&self, thread: &Thread) -> Result<(), RuntimeError> {
        match self {
            BranchTable::Swith {
                info,
                table,
                else_label,
            } => {
                let data: Vec<u8> = match info {
                    BranchTableExprInfo::Primitive(size) => {
                        thread.env.stack.pop(*size).map_err(|e| e.into())?
                    }
                    BranchTableExprInfo::String => {
                        let heap_address = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                        let data = thread
                            .runtime
                            .heap
                            .read(heap_address as usize, 16)
                            .expect("Heap Read should have succeeded");
                        let (length, rest) = extract_u64(&data)?;
                        let (capacity, rest) = extract_u64(rest)?;
                        let data = thread
                            .runtime
                            .heap
                            .read(heap_address as usize + 16, length as usize)
                            .expect("Heap Read should have succeeded");
                        data
                    }
                };
                match (table.iter().find(|(d, l)| d == &data), else_label) {
                    (None, None) => return Err(RuntimeError::IncorrectVariant),
                    (None, Some(else_label)) => {
                        let Some(idx) = thread.env.program.get(&else_label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        thread.env.program.cursor_set(idx);
                        Ok(())
                    }
                    (Some((_, label)), _) => {
                        let Some(idx) = thread.env.program.get(label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        thread.env.program.cursor_set(idx);
                        Ok(())
                    }
                }
            }
            BranchTable::Table { table, else_label } => {
                let variant = OpPrimitive::get_num8::<u64>(&thread.memory())?;
                match (table.iter().find(|(d, l)| d == &variant), else_label) {
                    (None, None) => return Err(RuntimeError::IncorrectVariant),
                    (None, Some(else_label)) => {
                        let Some(idx) = thread.env.program.get(&else_label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        thread.env.program.cursor_set(idx);
                        Ok(())
                    }
                    (Some((_, label)), _) => {
                        let Some(idx) = thread.env.program.get(label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        thread.env.program.cursor_set(idx);
                        Ok(())
                    }
                }
            }
        }
    }
}
