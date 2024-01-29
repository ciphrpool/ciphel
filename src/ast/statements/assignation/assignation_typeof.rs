use std::cell::Ref;

use super::{AssignValue, Assignation, Assignee};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::Either;
use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Assignation<Scope> {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_unit().into(),
        ))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for AssignValue<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
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
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
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
