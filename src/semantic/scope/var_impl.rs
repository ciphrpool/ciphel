
use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EType, SemanticError, TypeOf},
};

use super::{
    type_traits::{GetSubTypes, TypeChecking},
    BuildVar,
};
use crate::semantic::scope::scope::Scope;

#[derive(Debug, Clone, PartialEq, Copy, Eq, PartialOrd, Ord)]
pub enum VarState {
    Local,
    Global,
    Parameter,
    ClosureEnv,
}

impl Default for VarState {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub id: ID,
    pub type_sig: EType,
    pub state: VarState,
    pub is_declared: bool,
}

impl PartialOrd for Var {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.id.partial_cmp(&other.id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.state.partial_cmp(&other.state)
    }
}

impl Eq for Var {}

impl Ord for Var {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl CompatibleWith for Var {
    fn compatible_with<Other>(
        &self,
        other: &Other,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        self.type_sig.compatible_with(other, scope)
    }
}

impl TypeOf for Var {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        self.type_sig.type_of(&scope)
    }
}

impl BuildVar for Var {
    fn build_var(id: &ID, type_sig: &EType) -> Self {
        Self {
            id: id.clone(),
            type_sig: type_sig.clone(),
            state: VarState::Local,
            is_declared: false,
        }
    }
}
impl GetSubTypes for Var {
    fn get_nth(&self, n: &usize) -> Option<EType> {
        <EType as GetSubTypes>::get_nth(&self.type_sig, n)
    }
    fn get_field(&self, field_id: &ID) -> Option<EType> {
        <EType as GetSubTypes>::get_field(&self.type_sig, field_id)
    }
    fn get_item(&self) -> Option<EType> {
        <EType as GetSubTypes>::get_item(&self.type_sig)
    }
    fn get_field_offset(&self, field_id: &ID) -> Option<usize> {
        self.type_sig.get_field_offset(field_id)
    }

    fn get_inline_field_offset(&self, index: usize) -> Option<usize> {
        self.type_sig.get_inline_field_offset(index)
    }
}
impl TypeChecking for Var {
    fn is_iterable(&self) -> bool {
        <EType as TypeChecking>::is_iterable(&self.type_sig)
    }

    fn is_boolean(&self) -> bool {
        <EType as TypeChecking>::is_boolean(&self.type_sig)
    }
    fn is_callable(&self) -> bool {
        <EType as TypeChecking>::is_callable(&self.type_sig)
    }
    fn is_any(&self) -> bool {
        <EType as TypeChecking>::is_any(&self.type_sig)
    }
    fn is_addr(&self) -> bool {
        <EType as TypeChecking>::is_addr(&self.type_sig)
    }
    fn is_char(&self) -> bool {
        <EType as TypeChecking>::is_char(&self.type_sig)
    }
    fn is_indexable(&self) -> bool {
        <EType as TypeChecking>::is_indexable(&self.type_sig)
    }
    fn is_dotnum_indexable(&self) -> bool {
        <EType as TypeChecking>::is_dotnum_indexable(&self.type_sig)
    }
    fn is_map(&self) -> bool {
        <EType as TypeChecking>::is_map(&self.type_sig)
    }
    fn is_u64(&self) -> bool {
        <EType as TypeChecking>::is_u64(&self.type_sig)
    }
    fn is_unit(&self) -> bool {
        <EType as TypeChecking>::is_unit(&self.type_sig)
    }
    fn is_vec(&self) -> bool {
        <EType as TypeChecking>::is_vec(&self.type_sig)
    }
}
