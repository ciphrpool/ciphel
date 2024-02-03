use std::cell::Ref;

use super::{EventCondition, EventDef};

use crate::semantic::{
    scope::{
        static_types::StaticType, user_type_impl::UserType, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> TypeOf<Scope> for EventDef<Scope> {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for EventCondition {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
