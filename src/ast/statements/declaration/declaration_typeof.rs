use std::cell::Ref;

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Declaration<Scope> {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_unit()))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TypedVar {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.signature.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for DeclaredVar {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            DeclaredVar::Id(_) => Ok(EitherType::Static(Scope::StaticType::build_unit())),
            DeclaredVar::Typed(value) => value.type_of(&scope),
            DeclaredVar::Pattern(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PatternVar {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_unit()))
    }
}
