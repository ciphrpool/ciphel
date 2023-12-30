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
    ExpectCallable,
    ExpectEnum,
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
