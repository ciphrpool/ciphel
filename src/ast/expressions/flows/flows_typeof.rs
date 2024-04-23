use std::cell::Ref;

use crate::semantic::scope::scope_impl::Scope;
use crate::{
    ast::expressions::data::{VarID, Variable},
    p_num,
    semantic::{
        scope::{
            static_types::{NumberType, PrimitiveType, StaticType},
            type_traits::GetSubTypes,
            user_type_impl::UserType,
        },
        EType, Either, MergeType, Resolve, SemanticError, TypeOf,
    },
    vm::platform::Lib,
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, PatternExpr, TryExpr};

impl TypeOf for ExprFlow {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ExprFlow::If(value) => value.type_of(&scope),
            ExprFlow::Match(value) => value.type_of(&scope),
            ExprFlow::Try(value) => value.type_of(&scope),
            ExprFlow::Call(value) => value.type_of(&scope),
            ExprFlow::SizeOf(..) => Ok(p_num!(U64)),
        }
    }
}
impl TypeOf for IfExpr {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let then_branch_type = self.then_branch.type_of(&scope)?;
        then_branch_type.merge(&self.else_branch, scope)
    }
}

impl TypeOf for PatternExpr {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.expr.type_of(&scope)
    }
}
impl TypeOf for MatchExpr {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                pattern_type.merge(&else_type, scope)
            }
            None => Ok(pattern_type),
        }
    }
}
impl TypeOf for TryExpr {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let try_branch_type = self.try_branch.type_of(&scope)?;
        try_branch_type.merge(&self.else_branch, scope)
    }
}
impl TypeOf for FnCall {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match &self.fn_var {
            Variable::Var(VarID { id, .. }) => {
                let borrow = self.platform.as_ref().borrow();
                match borrow.as_ref() {
                    Some(api) => return api.type_of(scope),
                    None => {}
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
