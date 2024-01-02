use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EitherType, Resolve, SemanticError, TypeOf},
};

use super::{
    type_traits::{GetSubTypes, TypeChecking},
    BuildEvent, BuildVar, ScopeApi,
};

#[derive(Debug, Clone)]
pub struct Event {}

impl<Scope: ScopeApi<Event = Self>> CompatibleWith<Scope> for Event {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Event = Self>> TypeOf<Scope> for Event {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Event = Self>> BuildEvent<Scope> for Event {
    fn build_event(scope: &Scope, event: &crate::ast::statements::definition::EventDef) -> Self {
        todo!()
    }
}
