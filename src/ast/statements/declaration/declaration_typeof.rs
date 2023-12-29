use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl<Scope: ScopeApi> TypeOf<Scope> for Declaration {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(None)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TypedVar {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.signature.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for DeclaredVar {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            DeclaredVar::Id(_) => Ok(None),
            DeclaredVar::Typed(value) => value.type_of(scope),
            DeclaredVar::Pattern(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PatternVar {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(None)
    }
}
