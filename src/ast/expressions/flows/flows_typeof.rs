use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl TypeOf for ExprFlow {
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
impl TypeOf for IfExpr {
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
impl TypeOf for Pattern {
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
impl TypeOf for PatternExpr {
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
impl TypeOf for MatchExpr {
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
impl TypeOf for TryExpr {
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
impl TypeOf for FnCall {
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
