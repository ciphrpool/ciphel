use crate::e_static;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::StringType;
use crate::{
    p_num,
    semantic::{scope::static_types::StaticType, EType, MergeType, Resolve, SemanticError, TypeOf},
};

use super::{ExprFlow, FCall, IfExpr, MatchExpr, PatternExpr, TryExpr};

impl TypeOf for ExprFlow {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ExprFlow::If(value) => value.type_of(&scope),
            ExprFlow::Match(value) => value.type_of(&scope),
            ExprFlow::Try(value) => value.type_of(&scope),
            ExprFlow::FCall(value) => value.type_of(&scope),
            ExprFlow::SizeOf(..) => Ok(p_num!(U64)),
        }
    }
}
impl TypeOf for IfExpr {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let then_branch_type = self.then_branch.type_of(&scope)?;
        then_branch_type.merge(&self.else_branch, scope)
    }
}

impl TypeOf for PatternExpr {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.expr.type_of(&scope)
    }
}
impl TypeOf for MatchExpr {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
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
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for FCall {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(e_static!(StaticType::String(StringType())))
    }
}
