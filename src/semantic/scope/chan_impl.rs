use std::cell::Ref;

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EType, Either, SemanticError, TypeOf},
};

use super::{static_types::StaticType, user_type_impl::UserType, BuildChan, ScopeApi};

#[derive(Debug, Clone, PartialEq)]
pub struct Chan {}

impl<Scope: ScopeApi> CompatibleWith<Scope> for Chan {
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

impl<Scope: ScopeApi> TypeOf<Scope> for Chan {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> BuildChan<Scope> for Chan {
    fn build_chan(_id: &ID, _type_sig: &EType) -> Self {
        todo!()
    }
}
