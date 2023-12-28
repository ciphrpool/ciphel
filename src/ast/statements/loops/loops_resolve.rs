use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{ForIterator, ForLoop, Loop, WhileLoop};

impl<Scope: ScopeApi> Resolve<Scope> for Loop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForIterator {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForLoop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for WhileLoop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
