use std::cell::Ref;

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::Either;
use crate::semantic::{
    scope::{
        static_types::StaticType, user_type_impl::UserType, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Declaration<Scope> {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_unit().into(),
        ))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TypedVar {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.signature.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for DeclaredVar {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            DeclaredVar::Id(_) => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            DeclaredVar::Typed(value) => value.type_of(&scope),
            DeclaredVar::Pattern(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PatternVar {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_unit().into(),
        ))
    }
}
