use std::cell::Ref;

use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
    EType, Either, Resolve, SemanticError, TypeOf,
};

use super::{ForIterator, ForLoop, Loop, WhileLoop};

impl<Scope: ScopeApi> TypeOf<Scope> for Loop<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Loop::For(value) => value.type_of(&scope),
            Loop::While(value) => value.type_of(&scope),
            Loop::Loop(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ForIterator<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ForIterator::Id(value) => {
                let var = scope.find_var(value)?;
                var.type_of(&scope)
            }
            ForIterator::Vec(value) => value.type_of(&scope),
            ForIterator::Slice(value) => value.type_of(&scope),
            ForIterator::Receive { addr, timeout: _ } => addr.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ForLoop<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.scope.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for WhileLoop<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.scope.type_of(&scope)
    }
}
