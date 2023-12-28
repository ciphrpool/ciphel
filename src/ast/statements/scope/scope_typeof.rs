use crate::semantic::{EitherType, Resolve, ScopeApi, SemanticError, TypeOf};

use super::Scope;

impl<OuterScope: ScopeApi> TypeOf<OuterScope> for Scope {
    fn type_of(
        &self,
        scope: &OuterScope,
    ) -> Result<Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>, SemanticError>
    where
        OuterScope: ScopeApi,
        Self: Sized + Resolve<OuterScope>,
    {
        todo!()
    }
}
