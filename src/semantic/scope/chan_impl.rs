use std::cell::Ref;

use crate::semantic::scope::scope_impl::Scope;
use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EType, Either, SemanticError, TypeOf},
};

use super::{static_types::StaticType, user_type_impl::UserType, BuildChan};

#[derive(Debug, Clone, PartialEq)]
pub struct Chan {}

impl CompatibleWith for Chan {
    fn compatible_with<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        todo!()
    }
}

impl TypeOf for Chan {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl BuildChan for Chan {
    fn build_chan(_id: &ID, _type_sig: &EType) -> Self {
        todo!()
    }
}
