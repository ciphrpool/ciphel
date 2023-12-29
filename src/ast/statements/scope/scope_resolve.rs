use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError};

use super::Scope;

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope {
    type Output = ();
    type Context = Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>;
    fn resolve(&self, scope: &OuterScope, context: &Self::Context) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        let mut inner_scope = scope.child_scope()?;
        match self
            .instructions
            .iter()
            .find_map(|instruction| instruction.resolve(&inner_scope, &()).err())
        {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
