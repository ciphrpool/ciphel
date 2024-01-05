use super::Error;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for Error {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}
