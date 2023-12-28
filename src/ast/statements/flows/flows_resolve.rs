use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, Return, TryStat};

impl<Scope: ScopeApi> Resolve<Scope> for Flow {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Flow::If(value) => value.resolve(scope),
            Flow::Match(value) => value.resolve(scope),
            Flow::Try(value) => value.resolve(scope),
            Flow::Call(value) => value.resolve(scope),
            Flow::Return(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope)?;
        // TODO : check that condition is a boolean
        let _ = self.main_branch.resolve(scope)?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.expr.resolve(scope)?;

        let _ = {
            match self
                .patterns
                .iter()
                .find_map(|value| value.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.pattern.resolve(scope)?;
        let _ = self.scope.resolve(scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.try_branch.resolve(scope)?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for CallStat {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = scope.find_fn(&self.fn_id)?;
        let _ = self.params.resolve(scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Return {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Return::Unit => Ok(()),
            Return::Expr(value) => value.resolve(scope),
        }
    }
}
