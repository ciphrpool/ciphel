use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf};

use super::Error;

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
