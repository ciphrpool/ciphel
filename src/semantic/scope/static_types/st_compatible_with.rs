
use crate::semantic::scope::scope::Scope;
use crate::semantic::{CompatibleWith, Either, SemanticError, TypeOf};

use super::StaticType;

impl CompatibleWith for StaticType {
    fn compatible_with<Other>(
        &self,
        other: &Other,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            StaticType::Primitive(value) => value.compatible_with(other, scope),
            StaticType::Slice(value) => value.compatible_with(other, scope),
            StaticType::Vec(value) => value.compatible_with(other, scope),
            StaticType::StaticFn(value) => value.compatible_with(other, scope),
            StaticType::Closure(value) => value.compatible_with(other, scope),
            StaticType::Tuple(value) => value.compatible_with(other, scope),
            StaticType::Range(value) => value.compatible_with(other, scope),
            StaticType::Unit => {
                let other_type = other.type_of(&scope)?;
                if let Either::Static(other_type) = other_type {
                    if let StaticType::Unit = other_type.as_ref() {
                        return Ok(());
                    } else {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                } else {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
            StaticType::Address(value) => value.compatible_with(other, scope),
            StaticType::Map(value) => value.compatible_with(other, scope),
            StaticType::Any => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::String(value) => value.compatible_with(other, scope),
            StaticType::StrSlice(value) => value.compatible_with(other, scope),
        }
    }
}
