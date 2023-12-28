use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl TypeOf for Declaration {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for TypedVar {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for DeclaredVar {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for PatternVar {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
