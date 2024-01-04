use crate::semantic::{scope::ScopeApi, Resolve, SemanticError};

use super::Error;

impl<Scope: ScopeApi> Resolve<Scope> for Error {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        _scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}
