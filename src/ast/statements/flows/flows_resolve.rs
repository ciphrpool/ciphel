use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, Return, TryStat};

impl<Scope: ScopeApi> Resolve<Scope> for Flow {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for CallStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Return {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
