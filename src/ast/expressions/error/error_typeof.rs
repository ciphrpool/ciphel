use std::cell::Ref;

use super::Error;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_error()))
    }
}
