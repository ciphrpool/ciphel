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

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_error().into(),
        ))
    }
}
