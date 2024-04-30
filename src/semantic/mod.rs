use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    ops::Deref,
    rc::Rc,
};

use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::Scope;

use self::scope::{static_types::StaticType, user_type_impl::UserType};

pub mod scope;
pub mod utils;

pub type MutRc<T> = Rc<RefCell<T>>;

#[derive(Debug, Clone)]
pub enum SemanticError {
    NotResolvedYet,
    PlatformAPIOverriding,

    CantReturn,
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
    ExpectedLoop,
    ExpectedMovedClosure,

    UnknownVar(ID),
    UnknownType(ID),
    UnknownField,

    DoubleInfinitRange,

    IncorrectArguments,
    IncorrectStruct,
    IncorrectVariant,
    InvalidPattern,
    InvalidRange,

    IncompatibleTypes,
    IncompatibleOperation,
    IncompatibleOperands,
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub info: MutRc<Info>,
}

impl Metadata {
    pub fn context(&self) -> Option<EType> {
        let borrowed = self.info.as_ref().borrow();
        match borrowed.deref() {
            Info::Unresolved => None,
            Info::Resolved {
                context,
                signature: _,
            } => context.clone(),
        }
    }
    pub fn signature(&self) -> Option<EType> {
        let borrowed = self.info.as_ref().borrow();
        match borrowed.deref() {
            Info::Unresolved => None,
            Info::Resolved {
                context: _,
                signature,
            } => signature.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Info {
    Unresolved,
    Resolved {
        context: Option<EType>,
        signature: Option<EType>,
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

pub type EType = Either<UserType, StaticType>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessLevel {
    General,
    Direct,
    Backward(usize),
}

pub trait Resolve {
    type Output;
    type Context: Default;
    type Extra: Default;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait CompatibleWith {
    fn compatible_with<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf;
}

pub trait TypeOf {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized;
}

pub trait MergeType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf;
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
