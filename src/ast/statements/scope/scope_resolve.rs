use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::Scope;

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope {
    type Output = ();
    fn resolve(&self, scope: &OuterScope) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        todo!()
    }
}
