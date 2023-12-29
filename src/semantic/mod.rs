use std::rc::Rc;

use crate::ast::{statements::definition, utils::strings::ID};

use self::scope::ScopeApi;

pub mod scope;
pub mod utils;

#[derive(Debug, Clone)]
pub enum SemanticError {
    CantInferType,
    ExpectBoolean,
    ExpectIterable,
    UnknownField,
    IncorrectVariant,
    InvalidPattern,
    IncompatibleTypes,
    IncompatibleOperation,
    IncompatibleOperands,
}
#[derive(Debug, Clone, PartialEq)]
pub enum EitherType<User, Static> {
    Static(Static),
    User(User),
}

pub trait Resolve<Scope: ScopeApi> {
    type Output;
    type Context: Default;
    fn resolve(
        &self,
        scope: &Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait CompatibleWith<Scope: ScopeApi> {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait TypeOf<Scope: ScopeApi> {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>;
}

pub trait MergeType<Scope: ScopeApi> {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Other: TypeOf<Scope> + Resolve<Scope>;
}

pub trait BuildFn<Scope: ScopeApi> {
    fn build_fn(func: &definition::FnDef) -> Scope::Fn;
}

pub trait BuildType<Scope: ScopeApi> {
    fn build_type(type_sig: &definition::TypeDef) -> Scope::UserType;
}

pub trait BuildVar<Scope: ScopeApi> {
    fn build_var(id: &ID, type_sig: &EitherType<Scope::UserType, Scope::StaticType>) -> Scope::Var;
}
pub trait BuildChan<Scope: ScopeApi> {
    fn build_chan(id: &ID, type_sig: &EitherType<Scope::UserType, Scope::StaticType>)
        -> Scope::Var;
}

pub trait BuildEvent<Scope: ScopeApi> {
    fn build_event(scope: &Scope, event: &definition::FnDef) -> Scope::Var;
}
