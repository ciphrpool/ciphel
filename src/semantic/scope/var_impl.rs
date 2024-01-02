use crate::{
    ast::utils::strings::ID,
    semantic::{CompatibleWith, EitherType, Resolve, SemanticError, TypeOf},
};

use super::{
    static_type_impl::StaticType,
    type_traits::{GetSubTypes, TypeChecking},
    user_type_impl::UserType,
    BuildVar, ScopeApi,
};

#[derive(Debug, Clone)]
pub struct Var {
    pub id: ID,
    pub type_sig: EitherType<UserType, StaticType>,
}

impl<Scope: ScopeApi<Var = Self>> CompatibleWith<Scope> for Var {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<Var = Self>> TypeOf<Scope> for Var {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi<Var = Self>> Resolve<Scope> for Var {
    type Output = ();

    type Context = ();

    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
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
        }
    }
}
impl<Scope: ScopeApi<Var = Self>> GetSubTypes<Scope> for Var {}
impl<Scope: ScopeApi<Var = Self>> TypeChecking<Scope> for Var {}
