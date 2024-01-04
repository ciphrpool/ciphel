use crate::semantic::{
    scope::{
        type_traits::{TypeChecking},
        ScopeApi,
    }, EitherType, Resolve, SemanticError, TypeOf,
};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, TryStat};

impl<Scope: ScopeApi> Resolve<Scope> for Flow {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Flow::If(value) => value.resolve(scope, context),
            Flow::Match(value) => value.resolve(scope, context),
            Flow::Try(value) => value.resolve(scope, context),
            Flow::Call(value) => value.resolve(scope, &()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfStat {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope, &None)?;
        // check that condition is a boolean
        let condition_type = self.condition.type_of(scope)?;
        if !<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
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
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.expr.resolve(scope, &None)?;
        let expr_type = Some(self.expr.type_of(scope)?);

        for value in &self.patterns {
            let _ = value.resolve(scope, &expr_type)?;
        }
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &None)?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternStat {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let vars = self.pattern.resolve(scope, context)?;
        // create a scope and assign the pattern variable to it before resolving the expression
        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(vars.into_iter());
        let _ = self.scope.resolve(&mut inner_scope, context)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryStat {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.call.resolve(scope, &None)
    }
}
