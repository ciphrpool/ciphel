use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::Error;

impl<Scope: ScopeApi> Resolve<Scope> for Error {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
