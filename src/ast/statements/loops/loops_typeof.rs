use crate::semantic::{EType, Resolve, SemanticError, TypeOf};

use super::{ForLoop, Loop, WhileLoop};
use crate::semantic::scope::scope::ScopeManager;

impl TypeOf for Loop {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Loop::For(value) => value.type_of(&scope_manager, scope_id),
            Loop::While(value) => value.type_of(&scope_manager, scope_id),
            Loop::Loop(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl TypeOf for ForLoop {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.block.type_of(&scope_manager, scope_id)
    }
}

impl TypeOf for WhileLoop {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.block.type_of(&scope_manager, scope_id)
    }
}
