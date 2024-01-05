use std::cell::Ref;

use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EitherType, SemanticError, TypeOf},
};

use super::{BuildChan, ScopeApi};

#[derive(Debug, Clone)]
pub struct Chan {}

impl<Scope: ScopeApi<Chan = Self>> CompatibleWith<Scope> for Chan {
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

impl<Scope: ScopeApi<Chan = Self>> TypeOf<Scope> for Chan {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Chan = Self>> BuildChan<Scope> for Chan {
    fn build_chan(
        _id: &ID,
        _type_sig: &EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
    ) -> Self {
        todo!()
    }
}
