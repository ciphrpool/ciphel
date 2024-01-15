use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::ast::utils::strings::ID;

use self::scope::ScopeApi;

pub mod scope;
pub mod utils;

#[derive(Debug, Clone)]
pub enum SemanticError {
    NotResolvedYet,

    CantInferType,
    CantRegisterType,
    CantRegisterVar,

    ExpectedBoolean,
    ExpectedIterable,
    ExpectedIndexable,
    ExpectedCallable,
    ExpectedEnum,
    ExpectedStruct,
    ExpectedChannel,

    UnknownVar(ID),
    UnknownType,
    UnknownField,

    IncorrectStruct,
    IncorrectVariant,
    InvalidPattern,

    IncompatibleTypes,
    IncompatibleOperation,
    IncompatibleOperands,
}
#[derive(Debug, Clone, PartialEq)]
pub enum EitherType<User, Static> {
    Static(Rc<Static>),
    User(Rc<User>),
}

pub trait Resolve<Scope: ScopeApi> {
    type Output;
    type Context: Default;
    type Extra: Default;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait CompatibleWith<Scope: ScopeApi> {
    fn compatible_with<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait TypeOf<Scope: ScopeApi> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized;
}

pub trait MergeType<Scope: ScopeApi> {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
