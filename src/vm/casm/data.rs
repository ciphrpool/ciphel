use std::fmt;

use ulid::Ulid;

use crate::vm::{
    allocator::{heap::Heap, stack::Stack},
    stdio::StdIO,
    vm::{CasmMetadata, Executable, RuntimeError},
};

use super::CasmProgram;
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

impl<G: crate::GameEngineStaticFn + Clone> CasmMetadata<G> for Data {
    fn name(&self, stdio: &mut StdIO<G>, program: &CasmProgram, engine: &mut G) {
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
                        let label = program.get_label_name(e).unwrap_or("".into()).to_string();
                        label.to_string()
                    })
                    .collect();
                let arr = arr.join(", ");
                stdio.push_casm(engine, &format!("labels {}", arr))
            }
        }
    }
}
impl<G: crate::GameEngineStaticFn + Clone> Executable<G> for Data {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO<G>,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            Data::Serialized { data } => {
                let _ = stack.push_with(&data).map_err(|e| e.into())?;
                program.incr();
            }
            Data::Dump { data } => program.incr(),
            Data::Table { data } => program.incr(),
        }
        Ok(())
    }
}
