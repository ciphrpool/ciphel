use std::cell::Ref;

use crate::semantic::{
    EType, Resolve, SemanticError, TypeOf,
};

use super::{ForIterator, ForLoop, Loop, WhileLoop};
use crate::semantic::scope::scope::Scope;

impl TypeOf for Loop {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Loop::For(value) => value.type_of(&scope),
            Loop::While(value) => value.type_of(&scope),
            Loop::Loop(value) => value.type_of(&scope),
        }
    }
}
impl TypeOf for ForIterator {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.expr.type_of(scope)
    }
}
impl TypeOf for ForLoop {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.scope.type_of(&scope)
    }
}
impl TypeOf for WhileLoop {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.scope.type_of(&scope)
    }
}
