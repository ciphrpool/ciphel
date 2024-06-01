use std::cell::Ref;

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{
    scope::{static_types::StaticType},
    Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Either};

impl TypeOf for Declaration {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType>::build_unit().into(),
        ))
    }
}
impl TypeOf for TypedVar {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.signature.type_of(&scope)
    }
}
impl TypeOf for DeclaredVar {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            DeclaredVar::Id(_) => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            DeclaredVar::Typed(value) => value.type_of(&scope),
            DeclaredVar::Pattern(value) => value.type_of(&scope),
        }
    }
}
impl TypeOf for PatternVar {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType>::build_unit().into(),
        ))
    }
}
