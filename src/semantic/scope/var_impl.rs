use std::cell::{Ref, RefCell};

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EitherType, SemanticError, TypeOf},
};

use super::{
    static_types::StaticType,
    type_traits::{GetSubTypes, TypeChecking},
    user_type_impl::UserType,
    BuildVar, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub id: ID,
    pub type_sig: EitherType<UserType, StaticType>,
    pub captured: RefCell<bool>,
}

impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>>
    CompatibleWith<Scope> for Var
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        self.type_sig.compatible_with(other, scope)
    }
}

impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>> TypeOf<Scope>
    for Var
{
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        self.type_sig.type_of(&scope)
    }
}

impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>> BuildVar<Scope>
    for Var
{
    fn build_var(
        id: &ID,
        type_sig: &EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
    ) -> Self {
        Self {
            id: id.clone(),
            type_sig: type_sig.clone(),
            captured: RefCell::new(false),
        }
    }
}
impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>> GetSubTypes<Scope>
    for Var
{
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_nth(&self.type_sig, n)
    }
    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_field(
            &self.type_sig,
            field_id,
        )
    }
    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_item(&self.type_sig)
    }
}
impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>> TypeChecking<Scope>
    for Var
{
    fn is_iterable(&self) -> bool {
        <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_iterable(&self.type_sig)
    }
    fn is_channel(&self) -> bool {
        <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_channel(&self.type_sig)
    }
    fn is_boolean(&self) -> bool {
        <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_boolean(&self.type_sig)
    }
    fn is_callable(&self) -> bool {
        <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_callable(&self.type_sig)
    }
    fn is_any(&self) -> bool {
        <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_any(&self.type_sig)
    }
}
