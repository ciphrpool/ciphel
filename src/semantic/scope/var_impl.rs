use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, Either, SemanticError, TypeOf},
};

use super::{
    chan_impl::Chan,
    event_impl::Event,
    static_types::StaticType,
    type_traits::{GetSubTypes, TypeChecking},
    user_type_impl::UserType,
    BuildVar, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub id: ID,
    pub type_sig: Either<UserType, StaticType>,
    pub captured: RefCell<bool>,
    pub address: Option<Rc<Cell<usize>>>,
}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > CompatibleWith<Scope> for Var
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        self.type_sig.compatible_with(other, scope)
    }
}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TypeOf<Scope> for Var
{
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        self.type_sig.type_of(&scope)
    }
}

impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > BuildVar<Scope> for Var
{
    fn build_var(
        id: &ID,
        type_sig: &Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
    ) -> Self {
        Self {
            id: id.clone(),
            type_sig: type_sig.clone(),
            captured: RefCell::new(false),
            address: None,
        }
    }
}
impl<
        Scope: ScopeApi<
            StaticType = StaticType,
            UserType = UserType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > GetSubTypes<Scope> for Var
{
    fn get_nth(&self, n: &usize) -> Option<Either<Scope::UserType, Scope::StaticType>> {
        <Either<UserType, StaticType> as GetSubTypes<Scope>>::get_nth(&self.type_sig, n)
    }
    fn get_field(&self, field_id: &ID) -> Option<Either<Scope::UserType, Scope::StaticType>> {
        <Either<UserType, StaticType> as GetSubTypes<Scope>>::get_field(&self.type_sig, field_id)
    }
    fn get_item(&self) -> Option<Either<Scope::UserType, Scope::StaticType>> {
        <Either<UserType, StaticType> as GetSubTypes<Scope>>::get_item(&self.type_sig)
    }
}
impl<Scope: ScopeApi<Var = Self, StaticType = StaticType, UserType = UserType>> TypeChecking<Scope>
    for Var
{
    fn is_iterable(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_iterable(&self.type_sig)
    }
    fn is_channel(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_channel(&self.type_sig)
    }
    fn is_boolean(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_boolean(&self.type_sig)
    }
    fn is_callable(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_callable(&self.type_sig)
    }
    fn is_any(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_any(&self.type_sig)
    }
    fn is_addr(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_addr(&self.type_sig)
    }
    fn is_char(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_char(&self.type_sig)
    }
    fn is_indexable(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_indexable(&self.type_sig)
    }
    fn is_map(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_map(&self.type_sig)
    }
    fn is_u64(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_u64(&self.type_sig)
    }
    fn is_unit(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_unit(&self.type_sig)
    }
    fn is_vec(&self) -> bool {
        <Either<UserType, StaticType> as TypeChecking<Scope>>::is_vec(&self.type_sig)
    }
}
