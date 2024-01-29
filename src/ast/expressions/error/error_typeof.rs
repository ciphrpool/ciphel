use std::cell::Ref;

use super::Error;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Either, Resolve, SemanticError, TypeOf,
};

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for Error
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(Scope::StaticType::build_error().into()))
    }
}
