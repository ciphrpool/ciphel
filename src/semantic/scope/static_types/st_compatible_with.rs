use std::cell::Ref;

use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, user_type_impl::UserType, var_impl::Var, ScopeApi,
    },
    CompatibleWith, Either, SemanticError, TypeOf,
};

use super::StaticType;

impl<
        Scope: ScopeApi<StaticType = Self, UserType = UserType, Var = Var, Chan = Chan, Event = Event>,
    > CompatibleWith<Scope> for StaticType
{
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
        }
    }
}
