use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EitherType, Resolve, SemanticError, TypeOf},
};

use super::{
    type_traits::{GetSubTypes, TypeChecking},
    BuildChan, BuildVar, ScopeApi,
};

#[derive(Debug, Clone)]
pub struct Chan {}

impl<Scope: ScopeApi<Chan = Self>> CompatibleWith<Scope> for Chan {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Chan = Self>> TypeOf<Scope> for Chan {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Chan = Self>> BuildChan<Scope> for Chan {
    fn build_chan(
        id: &ID,
        type_sig: &EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
    ) -> Self {
        todo!()
    }
}
