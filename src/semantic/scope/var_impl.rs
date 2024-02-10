use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EType, Either, SemanticError, TypeOf},
    vm::allocator::stack::Offset,
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
    pub type_sig: EType,
    pub captured: RefCell<bool>,
    pub address: Cell<Option<Offset>>,
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for Var {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        self.type_sig.compatible_with(other, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Var {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        self.type_sig.type_of(&scope)
    }
}

impl<Scope: ScopeApi> BuildVar<Scope> for Var {
    fn build_var(id: &ID, type_sig: &EType) -> Self {
        Self {
            id: id.clone(),
            type_sig: type_sig.clone(),
            captured: RefCell::new(false),
            address: Cell::new(None),
        }
    }
}
impl GetSubTypes for Var {
    fn get_nth(&self, n: &usize) -> Option<EType> {
        <EType as GetSubTypes>::get_nth(&self.type_sig, n)
    }
    fn get_field(&self, field_id: &ID) -> Option<EType> {
        <EType as GetSubTypes>::get_field(&self.type_sig, field_id)
    }
    fn get_item(&self) -> Option<EType> {
        <EType as GetSubTypes>::get_item(&self.type_sig)
    }
    fn get_field_offset(&self, field_id: &ID) -> Option<usize> {
        self.type_sig.get_field_offset(field_id)
    }

    fn get_inline_field_offset(&self, index: usize) -> Option<usize> {
        self.type_sig.get_inline_field_offset(index)
    }
}
impl TypeChecking for Var {
    fn is_iterable(&self) -> bool {
        <EType as TypeChecking>::is_iterable(&self.type_sig)
    }
    fn is_channel(&self) -> bool {
        <EType as TypeChecking>::is_channel(&self.type_sig)
    }
    fn is_boolean(&self) -> bool {
        <EType as TypeChecking>::is_boolean(&self.type_sig)
    }
    fn is_callable(&self) -> bool {
        <EType as TypeChecking>::is_callable(&self.type_sig)
    }
    fn is_any(&self) -> bool {
        <EType as TypeChecking>::is_any(&self.type_sig)
    }
    fn is_addr(&self) -> bool {
        <EType as TypeChecking>::is_addr(&self.type_sig)
    }
    fn is_char(&self) -> bool {
        <EType as TypeChecking>::is_char(&self.type_sig)
    }
    fn is_indexable(&self) -> bool {
        <EType as TypeChecking>::is_indexable(&self.type_sig)
    }
    fn is_dotnum_indexable(&self) -> bool {
        <EType as TypeChecking>::is_dotnum_indexable(&self.type_sig)
    }
    fn is_map(&self) -> bool {
        <EType as TypeChecking>::is_map(&self.type_sig)
    }
    fn is_u64(&self) -> bool {
        <EType as TypeChecking>::is_u64(&self.type_sig)
    }
    fn is_unit(&self) -> bool {
        <EType as TypeChecking>::is_unit(&self.type_sig)
    }
    fn is_vec(&self) -> bool {
        <EType as TypeChecking>::is_vec(&self.type_sig)
    }
}
