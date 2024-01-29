use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::ast::utils::strings::ID;

use self::scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi};

pub mod scope;
pub mod utils;

#[derive(Debug, Clone)]
pub enum SemanticError {
    NotResolvedYet,
    PlatformAPIOverriding,

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

    IncorrectArguments,
    IncorrectStruct,
    IncorrectVariant,
    InvalidPattern,

    IncompatibleTypes,
    IncompatibleOperation,
    IncompatibleOperands,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub info: Rc<RefCell<Info>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Info {
    Unresolved,
    Resolved {
        context: Option<Either<UserType, StaticType>>,
        signature: Option<Either<UserType, StaticType>>,
    },
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            info: Rc::new(RefCell::new(Info::Unresolved)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Either<User, Static> {
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
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized;
}

pub trait MergeType<Scope: ScopeApi> {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
