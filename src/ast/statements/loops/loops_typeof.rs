use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

use super::{ForIterator, ForLoop, Loop, WhileLoop};

impl<Scope: ScopeApi> TypeOf<Scope> for Loop {
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
            Loop::For(value) => value.type_of(scope),
            Loop::While(value) => value.type_of(scope),
            Loop::Loop(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ForIterator {
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
            ForIterator::Id(value) => {
                let var = scope.find_var(value)?;
                var.type_of(scope)
            }
            ForIterator::Vec(value) => value.type_of(scope),
            ForIterator::Slice(value) => value.type_of(scope),
            ForIterator::Tuple(value) => value.type_of(scope),
            ForIterator::Receive { addr, timeout } => addr.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ForLoop {
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
        self.scope.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for WhileLoop {
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
        self.scope.type_of(scope)
    }
}
