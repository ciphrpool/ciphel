use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{ForIterator, ForLoop, Loop, WhileLoop};

impl TypeOf for Loop {
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
impl TypeOf for ForIterator {
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
impl TypeOf for ForLoop {
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
impl TypeOf for WhileLoop {
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
