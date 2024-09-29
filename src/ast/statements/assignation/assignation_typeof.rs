use super::{AssignValue, Assignation};
use crate::semantic::EType;
use crate::semantic::{scope::static_types::StaticType, Resolve, SemanticError, TypeOf};

impl TypeOf for Assignation {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::Unit))
    }
}
impl TypeOf for AssignValue {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            AssignValue::Block(value) => value.type_of(&scope_manager, scope_id),
            AssignValue::Expr(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}
