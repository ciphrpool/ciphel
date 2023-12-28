use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl<Scope: ScopeApi> Resolve<Scope> for ExprFlow {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ExprFlow::If(value) => value.resolve(scope),
            ExprFlow::Match(value) => value.resolve(scope),
            ExprFlow::Try(value) => value.resolve(scope),
            ExprFlow::Call(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfExpr {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope)?;
        // TODO : Check if condition is a boolean
        let _ = self.main_branch.resolve(scope)?;
        let _ = self.else_branch.resolve(scope)?;

        let main_branch_type = self.main_branch.type_of(scope)?;
        let _ = main_branch_type.compatible_with(&self.else_branch, scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Pattern {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Pattern::Primitive(value) => value.resolve(scope),
            Pattern::String(_) => Ok(()),
            Pattern::Enum { typename, value } => {
                let user_type = scope.find_type(typename)?;
                user_type.compatible_with(&(typename, value), scope)?;
                Ok(())
            }
            Pattern::UnionInline {
                typename,
                variant,
                vars,
            } => todo!(),
            Pattern::UnionFields {
                typename,
                variant,
                vars,
            } => todo!(),
            Pattern::StructInline { typename, vars } => todo!(),
            Pattern::StructFields { typename, vars } => todo!(),
            Pattern::Tuple(value) => todo!(),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternExpr {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.pattern.resolve(scope)?;
        // TODO : create a scope and assign the pattern variable to it before resolving the expression
        let _ = self.expr.resolve(scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchExpr {
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
                .find_map(|pattern| pattern.resolve(scope).err())
            {
                Some(e) => Err(e),
                None => Ok(()),
            }
        }?;
        let _ = self.else_branch.resolve(scope)?;

        let else_branch_type = self.else_branch.type_of(scope)?;

        let (maybe_err, _) =
            self.patterns
                .iter()
                .fold((None, else_branch_type), |mut previous, pattern| {
                    if let Err(e) = previous.1.compatible_with(pattern, scope) {
                        previous.0 = Some(e);
                        previous
                    } else {
                        match pattern.type_of(scope) {
                            Ok(pattern_type) => (None, pattern_type),
                            Err(err) => (Some(err), None),
                        }
                    }
                });
        if let Some(err) = maybe_err {
            return Err(err);
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryExpr {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.try_branch.resolve(scope)?;
        let _ = self.else_branch.resolve(scope)?;

        let try_branch_type = self.try_branch.type_of(scope)?;
        let _ = try_branch_type.compatible_with(&self.else_branch, scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FnCall {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let function = scope.find_fn(&self.fn_id)?;
        let _ = self.params.resolve(scope)?;

        let _ = function.compatible_with(&self.params, scope)?;

        Ok(())
    }
}
