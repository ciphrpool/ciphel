use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl<Scope: ScopeApi> TypeOf<Scope> for ExprFlow {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for IfExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Pattern {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PatternExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MatchExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TryExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for FnCall {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
