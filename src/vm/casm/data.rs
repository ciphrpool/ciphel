use std::fmt;

use ulid::Ulid;

use crate::vm::{
    allocator::{heap::Heap, stack::Stack},
    stdio::StdIO,
    vm::{CasmMetadata, Executable, RuntimeError},
};

use super::{alloc::ALLOC_SIZE_THRESHOLD, CasmProgram};
struct HexSlice<'a>(&'a [u8]);

impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            write!(f, "{:0>2x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    Serialized { data: Box<[u8]> },
    Dump { data: Box<[Box<[u8]>]> },
    Table { data: Box<[Ulid]> },
    // Get { label: Ulid, idx: Option<usize> },
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for Data {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            Data::Serialized { data } => {
                stdio.push_casm(engine, &format!("dmp 0x{}", HexSlice(data.as_ref())))
            }
            Data::Dump { data } => {
                let arr: Vec<String> = data.iter().map(|e| format!("0x{}", HexSlice(e))).collect();
                let arr = arr.join(", ");
                stdio.push_casm(engine, &format!("data {}", arr))
            }
            Data::Table { data } => {
                let arr: Vec<String> = data
                    .iter()
                    .map(|e| {
                        let label = program
                            .get_label_name(e)
                            .unwrap_or("".to_string().into())
                            .to_string();
                        label.to_string()
                    })
                    .collect();
                let arr = arr.join(", ");
                stdio.push_casm(engine, &format!("labels {}", arr))
            }
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            Data::Serialized { data } => {
                if data.len() > ALLOC_SIZE_THRESHOLD {
                    crate::vm::vm::CasmWeight::EXTREME
                } else {
                    crate::vm::vm::CasmWeight::LOW
                }
            }
            Data::Dump { data } => crate::vm::vm::CasmWeight::ZERO,
            Data::Table { data } => crate::vm::vm::CasmWeight::ZERO,
        }
    }
}
impl<G: crate::GameEngineStaticFn> Executable<G> for Data {
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
            Data::Serialized { data } => {
                let _ = stack.push_with(&data)?;
                program.incr();
            }
            Data::Dump { data } => program.incr(),
            Data::Table { data } => program.incr(),
        }
        Ok(())
    }
}
