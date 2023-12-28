use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, Return, TryStat};

impl<Scope: ScopeApi> TypeOf<Scope> for Flow {
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
impl<Scope: ScopeApi> TypeOf<Scope> for IfStat {
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
impl<Scope: ScopeApi> TypeOf<Scope> for MatchStat {
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
impl<Scope: ScopeApi> TypeOf<Scope> for PatternStat {
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
impl<Scope: ScopeApi> TypeOf<Scope> for TryStat {
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
impl<Scope: ScopeApi> TypeOf<Scope> for CallStat {
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
impl<Scope: ScopeApi> TypeOf<Scope> for Return {
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
