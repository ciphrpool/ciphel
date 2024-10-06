use super::{operation::OpPrimitive, CasmProgram};

use crate::{
    semantic::{scope::static_types::st_deserialize::extract_u64, AccessLevel},
    vm::{
        allocator::{
            heap::Heap,
            stack::{Offset, Stack},
        },
        stdio::StdIO,
        vm::{CasmMetadata, Executable, RuntimeError},
    },
};

use num_traits::ToBytes;
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

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Label {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        stdio.push_casm_label(engine, &self.name);
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Label {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        program.incr();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Call {
    From { label: Ulid, param_size: usize },
    Stack,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Call {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Call::From { label, param_size } => {
                let label = program
                    .get_label_name(label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("call {label} {param_size}"));
            }
            Call::Stack => stdio.push_casm(engine, "call"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for Call {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let (param_size, function_offset) = match self {
            Call::From { label, param_size } => {
                let Some(function_offset) = program.get(&label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                (*param_size, function_offset)
            }
            Call::Stack => {
                //let return_size = OpPrimitive::get_num8::<u64>(stack)? as usize;
                let param_size = OpPrimitive::get_num8::<u64>(stack)? as usize;
                let function_offset = OpPrimitive::get_num8::<u64>(stack)? as usize;
                (param_size, function_offset)
            }
        };
        if param_size != 0 {
            let data = stack.pop(param_size)?.to_owned();
            let _ = stack.frame(
                //return_size + 9 /* 8 bytes for the return size and 1 for wether the function returned something */,
                param_size,
                program.cursor + 1,
            )?;
            let _ = stack.write(Offset::FP(0), AccessLevel::Direct, &data)?;
        } else {
            let _ = stack.frame(
                //return_size + 9 /* 8 bytes for the return size and 1 for wether the function returned something */,
                param_size,
                program.cursor + 1,
            )?;
        }

        program.cursor_set(function_offset);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Goto {
    pub label: Option<Ulid>,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Goto {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self.label {
            Some(label) => {
                let label = program
                    .get_label_name(&label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("goto {label}"));
            }
            None => stdio.push_casm(engine, "goto"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for Goto {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self.label {
            Some(label) => {
                let Some(idx) = program.get(&label) else {
                    return Err(RuntimeError::CodeSegmentation);
                };
                program.cursor_set(idx);
                Ok(())
            }
            None => {
                let idx = OpPrimitive::get_num8::<u64>(stack)? as usize;
                program.cursor_set(idx);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchIf {
    pub else_label: Ulid,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for BranchIf {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        let label = program
            .get_label_name(&self.else_label)
            .unwrap_or("".to_string().into())
            .to_string();
        stdio.push_casm(engine, &format!("else {label}"));
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for BranchIf {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        let condition = OpPrimitive::get_bool(stack)?;

        let Some(else_label) = program.get(&self.else_label) else {
            return Err(RuntimeError::CodeSegmentation);
        };
        program.cursor_set(if condition {
            program.cursor + 1
        } else {
            else_label
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum BranchTable {
    Swith {
        size: Option<usize>,
        data_label: Option<Ulid>,
        else_label: Option<Ulid>,
    },
    Table {
        table_label: Option<Ulid>,
        else_label: Option<Ulid>,
    },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for BranchTable {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            BranchTable::Swith {
                size,
                data_label,
                else_label,
            } => {
                let data_label = match data_label {
                    Some(label) => program
                        .get_label_name(label)
                        .unwrap_or("".to_string().into())
                        .to_string(),
                    None => "".to_string().into(),
                };
                let else_label = match else_label {
                    Some(label) => program
                        .get_label_name(label)
                        .unwrap_or("".to_string().into())
                        .to_string(),
                    None => "".to_string().into(),
                };
                let size = match size {
                    Some(size) => format!("{size}"),
                    None => "".to_string().into(),
                };
                stdio.push_casm(engine, &format!("table {data_label} {else_label} {size}"));
            }
            BranchTable::Table {
                table_label,
                else_label,
            } => {
                let table_label = match table_label {
                    Some(label) => program
                        .get_label_name(label)
                        .unwrap_or("".to_string().into())
                        .to_string(),
                    None => "".to_string().into(),
                };
                let else_label = match else_label {
                    Some(label) => program
                        .get_label_name(label)
                        .unwrap_or("".to_string().into())
                        .to_string(),
                    None => "".to_string().into(),
                };
                stdio.push_casm(engine, &format!("table {table_label} {else_label}"));
            }
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for BranchTable {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            BranchTable::Swith {
                data_label,
                size,
                else_label,
            } => {
                let data_offset = match data_label {
                    Some(label) => {
                        let Some(data_offset) = program.get(&label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        data_offset
                    }
                    None => {
                        let data_offset = OpPrimitive::get_num8::<u64>(stack)? as usize;
                        data_offset
                    }
                };
                let data = match size {
                    Some(size) => stack.pop(*size)?.to_vec(),
                    None => {
                        let heap_address = OpPrimitive::get_num8::<u64>(stack)?;
                        let data = heap
                            .read(heap_address as usize, 16)
                            .expect("Heap Read should have succeeded");
                        let (length, rest) = extract_u64(&data)?;
                        let (_capacity, _rest) = extract_u64(rest)?;
                        let data = heap
                            .read(heap_address as usize + 16, length as usize)
                            .expect("Heap Read should have succeeded");
                        data
                    }
                };
                let data = data.into();
                let mut found_idx = None;
                for (idx, datum) in program.data_at_offset(data_offset)?.iter().enumerate() {
                    if datum == &data {
                        found_idx = Some(idx);
                        break;
                    }
                }

                match found_idx {
                    Some(idx) => {
                        let _ = stack.push_with(&(idx as u64).to_le_bytes())?;
                        program.incr();
                    }
                    None => {
                        if let Some(else_label) = else_label {
                            let Some(idx) = program.get(&else_label) else {
                                return Err(RuntimeError::CodeSegmentation);
                            };
                            program.cursor_set(idx);
                        } else {
                            return Err(RuntimeError::IncorrectVariant);
                        }
                    }
                }

                Ok(())
            }
            BranchTable::Table {
                table_label,
                else_label,
            } => {
                let table_offset = match table_label {
                    Some(label) => {
                        let Some(table_offset) = program.get(&label) else {
                            return Err(RuntimeError::CodeSegmentation);
                        };
                        table_offset
                    }
                    None => {
                        let table_offset = OpPrimitive::get_num8::<u64>(stack)? as usize;
                        table_offset
                    }
                };

                let variant = OpPrimitive::get_num8::<u64>(stack)?;

                let mut found_offset = None;
                for (idx, label) in program.table_at_offset(table_offset)?.iter().enumerate() {
                    let Some(instr_offset) = program.get(label) else {
                        return Err(RuntimeError::CodeSegmentation);
                    };
                    if variant == idx as u64 {
                        found_offset = Some(instr_offset);
                        break;
                    }
                }

                match found_offset {
                    Some(idx) => {
                        program.cursor_set(idx);
                    }
                    None => {
                        if let Some(else_label) = else_label {
                            let Some(idx) = program.get(&else_label) else {
                                return Err(RuntimeError::CodeSegmentation);
                            };
                            program.cursor_set(idx);
                        } else {
                            return Err(RuntimeError::IncorrectVariant);
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BranchTry {
    StartTry { else_label: Ulid },
    EndTry,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for BranchTry {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            BranchTry::StartTry { else_label } => {
                let label = program
                    .get_label_name(&else_label)
                    .unwrap_or("".to_string().into())
                    .to_string();
                stdio.push_casm(engine, &format!("try_else {label}"));
            }
            BranchTry::EndTry => stdio.push_casm(engine, &format!("try_end")),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::ZERO
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for BranchTry {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            BranchTry::StartTry { else_label } => {
                program.push_catch(else_label);
            }
            BranchTry::EndTry => {
                program.pop_catch();
            }
        }
        program.incr();
        Ok(())
    }
}
