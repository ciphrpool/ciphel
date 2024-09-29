use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::EType;
use crate::semantic::{scope::static_types::StaticType, Resolve, SemanticError, TypeOf};

impl TypeOf for Declaration {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::Unit))
    }
}
impl TypeOf for TypedVar {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.signature.type_of(&scope_manager, scope_id)
    }
}
impl TypeOf for DeclaredVar {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            DeclaredVar::Id { .. } => Ok(EType::Static(StaticType::Unit)),
            DeclaredVar::Typed(value) => value.type_of(&scope_manager, scope_id),
            DeclaredVar::Pattern(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}
impl TypeOf for PatternVar {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::Unit))
    }
}
