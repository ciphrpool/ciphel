use std::cell::Ref;

use super::Error;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EType;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
    Either, Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_error().into(),
        ))
    }
}
