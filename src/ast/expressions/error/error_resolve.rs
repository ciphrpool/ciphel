use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::Error;

impl Resolve for Error {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
