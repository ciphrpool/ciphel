use crate::{
    semantic::{
        scope::static_types::{PrimitiveType, StaticType},
        EitherType,
    },
    vm::{
        allocator::{Memory, MemoryAddress},
        vm::{Executable, RuntimeError},
    },
};

use super::{serialize::Serialized, Strips};

#[derive(Debug, Clone)]
pub struct IfSrip {
    boolean_expr: Serialized,
    then_strips: Strips,
    else_strips: Strips,
}

impl Executable for IfSrip {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        // let data = match self.boolean_expr {
        //     MemoryAddress::Heap {
        //         address,
        //         offset,
        //         size,
        //     } => memory
        //         .heap
        //         .read(address, offset, size)
        //         .map_err(|e| e.into())?,
        //     MemoryAddress::Stack { offset, size } => {
        //         memory.stack.read(offset, size).map_err(|e| e.into())?
        //     }
        // };
        match &self.boolean_expr.data_type {
            EitherType::Static(data_type) => match data_type.as_ref() {
                StaticType::Primitive(PrimitiveType::Bool) => {
                    let condition = self
                        .boolean_expr
                        .data
                        .first()
                        .map_or(false, |byte| *byte != 0);
                    if condition {
                        self.then_strips.execute(memory)
                    } else {
                        self.else_strips.execute(memory)
                    }
                }
                _ => Err(RuntimeError::Default),
            },
            _ => Err(RuntimeError::Default),
        }
    }
}
