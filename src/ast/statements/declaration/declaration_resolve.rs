use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl Resolve for Declaration {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for TypedVar {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for DeclaredVar {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for PatternVar {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
