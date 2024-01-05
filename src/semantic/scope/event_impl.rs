use std::cell::Ref;

use crate::semantic::{CompatibleWith, EitherType, SemanticError, TypeOf};

use super::{BuildEvent, ScopeApi};

#[derive(Debug, Clone)]
pub struct Event {}

impl<Scope: ScopeApi<Event = Self>> CompatibleWith<Scope> for Event {
    fn compatible_with<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Event = Self>> TypeOf<Scope> for Event {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Event = Self>> BuildEvent<Scope> for Event {
    fn build_event(
        _scope: &Ref<Scope>,
        _event: &crate::ast::statements::definition::EventDef,
    ) -> Self {
        todo!()
    }
}
