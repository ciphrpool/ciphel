use std::cell::Ref;

use super::{AssignValue, Assignation, Assignee};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Assignation<Scope> {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_unit().into()))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for AssignValue<Scope> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            AssignValue::Scope(value) => value.type_of(&scope),
            AssignValue::Expr(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Assignee<Scope> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Assignee::Variable(value) => value.type_of(&scope),
            Assignee::PtrAccess(value) => value.type_of(&scope),
        }
    }
}
