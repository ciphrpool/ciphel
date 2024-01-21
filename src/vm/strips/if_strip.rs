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

use super::{operation::OpPrimitive, serialize::Serialized, Strips};

#[derive(Debug, Clone)]
pub struct IfSrip {
    then_strips: Strips,
    else_strips: Strips,
}

impl Executable for IfSrip {
    fn execute(&self, memory: &Memory) -> Result<(), RuntimeError> {
        let condition = OpPrimitive::get_bool(memory)?;
        if condition {
            self.then_strips.execute(memory)
        } else {
            self.else_strips.execute(memory)
        }
    }
}
