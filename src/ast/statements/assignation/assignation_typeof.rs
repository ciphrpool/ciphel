use crate::semantic::{Resolve, ScopeApi, SemanticError, TypeOf};

use super::{AssignValue, Assignation, Assignee};

impl TypeOf for Assignation {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for AssignValue {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}

impl TypeOf for Assignee {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
