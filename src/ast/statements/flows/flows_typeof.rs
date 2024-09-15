use super::{CallStat, Flow, IfStat, MatchStat, TryStat};
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::EType;
use crate::semantic::{scope::static_types::StaticType, MergeType, Resolve, SemanticError, TypeOf};

impl TypeOf for Flow {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Flow::If(value) => value.type_of(&scope_manager, scope_id),
            Flow::Match(value) => value.type_of(&scope_manager, scope_id),
            Flow::Try(value) => value.type_of(&scope_manager, scope_id),
            Flow::Call(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}
impl TypeOf for IfStat {
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
impl TypeOf for MatchStat {
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

impl TypeOf for TryStat {
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

impl TypeOf for CallStat {
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
