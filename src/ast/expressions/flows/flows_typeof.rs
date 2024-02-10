use std::cell::Ref;

use crate::{
    ast::expressions::data::{VarID, Variable},
    semantic::{
        scope::{
            static_types::StaticType, type_traits::GetSubTypes, user_type_impl::UserType, ScopeApi,
        },
        EType, Either, MergeType, Resolve, SemanticError, TypeOf,
    },
    vm::platform::api::PlatformApi,
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, PatternExpr, TryExpr};

impl<Scope: ScopeApi> TypeOf<Scope> for ExprFlow<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ExprFlow::If(value) => value.type_of(&scope),
            ExprFlow::Match(value) => value.type_of(&scope),
            ExprFlow::Try(value) => value.type_of(&scope),
            ExprFlow::Call(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for IfExpr<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let then_branch_type = self.then_branch.type_of(&scope)?;
        then_branch_type.merge(&self.else_branch, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for PatternExpr<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.expr.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MatchExpr<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let pattern_type = {
            if let Some(res) = self.patterns.first() {
                let mut res = res.type_of(&scope)?;
                if self.patterns.len() > 1 {
                    for pattern in &self.patterns {
                        let pattern_type = pattern.type_of(&scope)?;
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
        let else_type = self.else_branch.type_of(&scope)?;
        pattern_type.merge(&else_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TryExpr<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let try_branch_type = self.try_branch.type_of(&scope)?;
        try_branch_type.merge(&self.else_branch, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for FnCall<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match &self.fn_var {
            Variable::Var(VarID { id, .. }) => {
                if let Some(api) = PlatformApi::from(id) {
                    return Ok(api.returns::<Scope>());
                }
            }
            _ => {}
        }

        let fn_var_type = self.fn_var.type_of(&scope)?;
        let Some(return_type) = <EType as GetSubTypes>::get_return(&fn_var_type) else {
            return Err(SemanticError::CantInferType);
        };

        Ok(return_type)
    }
}
