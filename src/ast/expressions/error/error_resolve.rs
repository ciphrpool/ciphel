use super::Error;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for Error {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::TryParse, semantic::scope::scope_impl::Scope};

    use super::*;

    #[test]
    fn valid_error() {
        let error = Error::parse("error".into()).unwrap().1;
        let scope = Scope::new();
        let res = error.resolve(&scope, &(), &());
        assert!(res.is_ok());
    }
}
