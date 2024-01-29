use std::cell::Ref;

use super::{EventCondition, EventDef};

use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for EventDef<Scope>
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for EventCondition
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
