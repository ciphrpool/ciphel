use std::cell::Ref;

use crate::semantic::{CompatibleWith, EType, SemanticError, TypeOf};

use super::{BuildEvent};
use crate::semantic::scope::scope_impl::Scope;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {}

impl CompatibleWith for Event {
    fn compatible_with<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        todo!()
    }
}

impl TypeOf for Event {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl BuildEvent for Event {
    fn build_event(
        _scope: &Ref<Scope>,
        _event: &crate::ast::statements::definition::EventDef,
    ) -> Self {
        todo!()
    }
}
