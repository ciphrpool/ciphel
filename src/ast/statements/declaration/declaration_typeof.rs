use std::cell::Ref;

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::Either;
use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for Declaration<Scope>
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(Scope::StaticType::build_unit().into()))
    }
}
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for TypedVar
{
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.signature.type_of(&scope)
    }
}
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for DeclaredVar
{
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            DeclaredVar::Id(_) => Ok(Either::Static(Scope::StaticType::build_unit().into())),
            DeclaredVar::Typed(value) => value.type_of(&scope),
            DeclaredVar::Pattern(value) => value.type_of(&scope),
        }
    }
}
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for PatternVar
{
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::Either<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(Scope::StaticType::build_unit().into()))
    }
}
