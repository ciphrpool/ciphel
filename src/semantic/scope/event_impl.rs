use std::cell::Ref;

use crate::semantic::{CompatibleWith, Either, SemanticError, TypeOf};

use super::{
    chan_impl::Chan, static_types::StaticType, user_type_impl::UserType, var_impl::Var, BuildEvent,
    ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Event {}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Self,
        >,
    > CompatibleWith<Scope> for Event
{
    fn compatible_with<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Self,
        >,
    > TypeOf<Scope> for Event
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Self,
        >,
    > BuildEvent<Scope> for Event
{
    fn build_event(
        _scope: &Ref<Scope>,
        _event: &crate::ast::statements::definition::EventDef<Scope>,
    ) -> Self {
        todo!()
    }
}
