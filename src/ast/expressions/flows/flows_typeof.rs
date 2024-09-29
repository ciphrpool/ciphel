use crate::{
    p_num,
    semantic::{EType, MergeType, Resolve, SemanticError, TypeOf},
};

use super::{ExprFlow, IfExpr, MatchExpr, TryExpr};

impl TypeOf for ExprFlow {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ExprFlow::If(value) => value.type_of(&scope_manager, scope_id),
            ExprFlow::Match(value) => value.type_of(&scope_manager, scope_id),
            ExprFlow::Try(value) => value.type_of(&scope_manager, scope_id),
            ExprFlow::SizeOf(..) => Ok(p_num!(U64)),
        }
    }
}
impl TypeOf for IfExpr {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.then_branch.type_of(&scope_manager, scope_id)
    }
}

impl TypeOf for MatchExpr {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let Some(Ok(case_type)) = (match &self.cases {
            super::Cases::Primitive { cases } => cases
                .first()
                .map(|case| case.block.type_of(scope_manager, scope_id)),
            super::Cases::String { cases } => cases
                .first()
                .map(|case| case.block.type_of(scope_manager, scope_id)),
            super::Cases::Enum { cases } => cases
                .first()
                .map(|case| case.block.type_of(scope_manager, scope_id)),
            super::Cases::Union { cases } => cases
                .first()
                .map(|case| case.block.type_of(scope_manager, scope_id)),
        }) else {
            return Err(SemanticError::CantInferType(
                "of this match expression".to_string(),
            ));
        };
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope_manager, scope_id)?;
                case_type.merge(&else_type, scope_manager, scope_id)
            }
            None => Ok(case_type),
        }
    }
}
impl TypeOf for TryExpr {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
