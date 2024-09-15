use crate::semantic::scope::scope::ScopeManager;
use crate::{
    ast::{expressions::data::Data, utils::strings::ID},
    semantic::{
        scope::{
            static_types::{AddrType, StaticType},
            type_traits::{OperandMerging, TypeChecking},
        },
        CompatibleWith, EType, MergeType, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, DeserializeFrom, Printer, RuntimeError},
    },
};

impl CompatibleWith for EType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        match (self, other) {
            (EType::Static(x), EType::Static(y)) => x.compatible_with(y, scope_manager, scope_id),
            (
                EType::User {
                    id: id_x,
                    size: size_x,
                },
                EType::User {
                    id: id_y,
                    size: size_y,
                },
            ) => (*id_x == *id_y && *size_x == *size_y)
                .then(|| ())
                .ok_or(SemanticError::IncompatibleTypes),
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl SizeOf for EType {
    fn size_of(&self) -> usize {
        match self {
            EType::Static(value) => value.size_of(),
            EType::User { id, size } => todo!(),
        }
    }
}

impl MergeType for EType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError> {
        match (self, other) {
            (EType::Static(x), EType::Static(y)) => x.merge(y, scope_manager, scope_id),
            (
                EType::User {
                    id: id_x,
                    size: size_x,
                },
                EType::User {
                    id: id_y,
                    size: size_y,
                },
            ) => {
                if *id_x != *id_y || *size_x != *size_y {
                    return Err(SemanticError::IncompatibleTypes);
                }
                return Ok(EType::User {
                    id: *id_x,
                    size: *size_x,
                });
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl DeserializeFrom for EType {
    type Output = Data;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            EType::Static(value) => value.deserialize_from(bytes),
            EType::User { id, size } => todo!(),
        }
    }
}

impl Printer for EType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            EType::Static(value) => value.build_printer(instructions),
            EType::User { id, size } => todo!(),
        }
    }
}
