use crate::semantic::{
    scope::{type_traits::GetSubTypes, ScopeApi},
    EitherType, MergeType, Resolve, SemanticError, TypeOf,
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, PatternExpr, TryExpr};

impl<Scope: ScopeApi> TypeOf<Scope> for ExprFlow {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ExprFlow::If(value) => value.type_of(scope),
            ExprFlow::Match(value) => value.type_of(scope),
            ExprFlow::Try(value) => value.type_of(scope),
            ExprFlow::Call(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for IfExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let main_branch_type = self.main_branch.type_of(scope)?;
        main_branch_type.merge(&self.else_branch, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for PatternExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.expr.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MatchExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let pattern_type = {
            if let Some(res) = self.patterns.first() {
                let mut res = res.type_of(scope)?;
                if self.patterns.len() > 1 {
                    for pattern in &self.patterns {
                        let pattern_type = pattern.type_of(scope)?;
                        res = res.merge(&pattern_type, scope)?;
                    }
                }
                Some(res)
            } else {
                None
            }
        };
        let Some(pattern_type) = pattern_type else {
            return Err(SemanticError::CantInferType);
        };
        let else_type = self.else_branch.type_of(scope)?;
        pattern_type.merge(&else_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TryExpr {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let try_branch_type = self.try_branch.type_of(scope)?;
        try_branch_type.merge(&self.else_branch, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for FnCall {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let fn_var_type = self.fn_var.type_of(scope)?;
        let Some(return_type) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_return(&fn_var_type) else {
            return Err(SemanticError::CantInferType);
        };

        Ok(return_type)
    }
}
