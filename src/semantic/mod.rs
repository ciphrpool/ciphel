use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::Scope;

use self::scope::{static_types::StaticType, user_type_impl::UserType};

pub mod scope;
pub mod utils;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcRwLock<T> = Arc<RwLock<T>>;

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
    ExpectedLoop,
    ExpectedMovedClosure,
    ExpectedLeftExpression,

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

    ConcurrencyError,
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub info: Info,
}

impl Metadata {
    pub fn context(&self) -> Option<EType> {
        match &self.info {
            Info::Unresolved => None,
            Info::Resolved {
                context,
                signature: _,
            } => context.clone(),
        }
    }
    pub fn signature(&self) -> Option<EType> {
        match &self.info {
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
            info: Info::Unresolved,
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

impl AccessLevel {
    pub fn name(&self) -> String {
        match self {
            AccessLevel::General => ", 0".to_string().into(),
            AccessLevel::Direct => "".to_string().into(),
            AccessLevel::Backward(n) => format!(", -{n}"),
        }
    }
}
pub trait Resolve {
    type Output;
    type Context: Default;
    type Extra: Default;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait CompatibleWith {
    fn compatible_with<Other>(
        &self,
        other: &Other,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf;
}

pub trait TypeOf {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized;
}

pub trait MergeType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf;
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
