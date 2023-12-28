use crate::semantic::{EitherType, Resolve, ScopeApi, SemanticError, TypeOf};

use super::Scope;

impl TypeOf for Scope {
    fn type_of<OuterScope>(
        &self,
        scope: &OuterScope,
    ) -> Result<Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>, SemanticError>
    where
        OuterScope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
