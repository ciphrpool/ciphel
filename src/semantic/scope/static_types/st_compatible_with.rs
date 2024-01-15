use std::cell::Ref;

use crate::{
    ast,
    semantic::{
        scope::{user_type_impl::UserType, BuildStaticType, ScopeApi},
        CompatibleWith, EitherType, SemanticError, TypeOf,
    },
};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, StaticType, TupleType,
    VecType,
};

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> CompatibleWith<Scope> for StaticType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            StaticType::Primitive(value) => value.compatible_with(other, scope),
            StaticType::Slice(value) => value.compatible_with(other, scope),
            StaticType::Vec(value) => value.compatible_with(other, scope),
            StaticType::Fn(value) => value.compatible_with(other, scope),
            StaticType::Chan(value) => value.compatible_with(other, scope),
            StaticType::Tuple(value) => value.compatible_with(other, scope),
            StaticType::Unit => {
                let other_type = other.type_of(&scope)?;
                if let EitherType::Static(other_type) = other_type {
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
        }
    }
}
