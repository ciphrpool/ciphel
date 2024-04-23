use super::Error;
use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::{MutRc, Resolve, SemanticError};


impl Resolve for Error {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
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
        assert!(res.is_ok(), "{:?}", res);
    }
}
