use crate::semantic::{EitherType, Resolve, SemanticError, TypeOf};

use super::Error;

impl TypeOf for Error {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: crate::semantic::ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
