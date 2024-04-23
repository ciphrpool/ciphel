use std::cell::Ref;

use super::Error;
use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EType;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType},
    Either, Resolve, SemanticError, TypeOf,
};

impl TypeOf for Error {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType>::build_error().into(),
        ))
    }
}
