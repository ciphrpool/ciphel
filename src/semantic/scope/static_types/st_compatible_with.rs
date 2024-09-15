use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::{CompatibleWith, SemanticError, TypeOf};

use super::StaticType;

impl CompatibleWith for StaticType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        match (self, other) {
            (StaticType::Primitive(x), StaticType::Primitive(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Slice(x), StaticType::Slice(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Map(x), StaticType::Map(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Vec(x), StaticType::Vec(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Range(x), StaticType::Range(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::String(x), StaticType::String(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::StrSlice(x), StaticType::StrSlice(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Closure(x), StaticType::Closure(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::StaticFn(x), StaticType::StaticFn(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Tuple(x), StaticType::Tuple(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Unit, StaticType::Unit) => Ok(()),
            (StaticType::Any, _) => Ok(()),
            (StaticType::Error, _) => Ok(()),
            (StaticType::Address(x), StaticType::Address(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}
