use std::cell::Ref;

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, Either, SemanticError, TypeOf},
};

use super::{
    event_impl::Event, static_types::StaticType, user_type_impl::UserType, var_impl::Var,
    BuildChan, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Chan {}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Self,
            Event = Event,
        >,
    > CompatibleWith<Scope> for Chan
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
            Chan = Self,
            Event = Event,
        >,
    > TypeOf<Scope> for Chan
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
            Chan = Self,
            Event = Event,
        >,
    > BuildChan<Scope> for Chan
{
    fn build_chan(
        _id: &ID,
        _type_sig: &Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
    ) -> Self {
        todo!()
    }
}
