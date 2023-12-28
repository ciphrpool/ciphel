use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::Scope;

impl Resolve for Scope {
    fn resolve<OuterScope>(&self, scope: &OuterScope) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        todo!()
    }
}
