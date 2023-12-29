use crate::semantic::{EitherType, Resolve, ScopeApi, SemanticError};

use super::Scope;

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope {
    type Output = ();
    type Context = Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>;
    fn resolve(&self, scope: &OuterScope, context: &Self::Context) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        match self
            .instructions
            .iter()
            .find_map(|instruction| instruction.resolve(scope, &()).err())
        {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
