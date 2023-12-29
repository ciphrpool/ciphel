use crate::semantic::{
    CompatibleWith, EitherType, Resolve, RetrieveTypeInfo, ScopeApi, SemanticError, TypeOf,
};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, Return, TryStat};

impl<Scope: ScopeApi> Resolve<Scope> for Flow {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Flow::If(value) => value.resolve(scope, context),
            Flow::Match(value) => value.resolve(scope, context),
            Flow::Try(value) => value.resolve(scope, context),
            Flow::Call(value) => value.resolve(scope, context),
            Flow::Return(value) => value.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfStat {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope, &None)?;
        // check that condition is a boolean
        let condition_type = self.condition.type_of(scope)?;
        if !<Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> as RetrieveTypeInfo<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectBoolean);
        }

        let _ = self.main_branch.resolve(scope, &None)?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &None)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchStat {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.expr.resolve(scope, &None)?;
        let expr_type = self.expr.type_of(scope)?;
        let _ = {
            match self
                .patterns
                .iter()
                .find_map(|value| value.resolve(scope, &expr_type).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &None)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternStat {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let vars = self.pattern.resolve(scope, context)?;
        // create a scope and assign the pattern variable to it before resolving the expression
        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(vars.into_iter());
        let _ = self.scope.resolve(&inner_scope, context)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryStat {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.try_branch.resolve(scope, &None)?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &None)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for CallStat {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let func = scope.find_fn(&self.fn_id)?;
        let _ = {
            match self.params.iter().enumerate().find_map(|(index, expr)| {
                let param_context = func.get_nth(&index);
                expr.resolve(scope, &param_context).err()
            }) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        let _ = func.compatible_with(&self.params, scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Return {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Return::Unit => Ok(()),
            Return::Expr(value) => value.resolve(scope, &None),
        }
    }
}
