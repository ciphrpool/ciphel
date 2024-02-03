use std::cell::Ref;

use crate::semantic::{CompatibleWith, Either, SemanticError, TypeOf};

use super::{
    static_types::StaticType, user_type_impl::UserType, BuildEvent,
    ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {}

impl<Scope: ScopeApi> CompatibleWith<Scope> for Event {
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

impl<Scope: ScopeApi> TypeOf<Scope> for Event {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> BuildEvent<Scope> for Event {
    fn build_event(
        _scope: &Ref<Scope>,
        _event: &crate::ast::statements::definition::EventDef<Scope>,
    ) -> Self {
        todo!()
    }
}
