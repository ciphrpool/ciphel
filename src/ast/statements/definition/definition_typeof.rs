use super::{
    EventCondition, EventDef,
};

use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for EventDef {
    fn type_of(
        &self,
        _scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for EventCondition {
    fn type_of(
        &self,
        _scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
