use crate::semantic::{EitherType, Resolve, ScopeApi, SemanticError, TypeOf};

use super::Error;

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: crate::semantic::ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
