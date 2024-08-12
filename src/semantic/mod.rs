use std::sync::{Arc, Mutex, RwLock};

use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::Scope;

use self::scope::{static_types::StaticType, user_type_impl::UserType};

use thiserror::Error;
pub mod scope;
pub mod utils;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcRwLock<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    #[error("NotResolvedYet")]
    NotResolvedYet,
    #[error("PlatformAPIOverriding")]
    PlatformAPIOverriding,

    #[error("cannot infer type {0}")]
    CantInferType(String),

    #[error("expected a boolean")]
    ExpectedBoolean,
    #[error("expected something iterable")]
    ExpectedIterable,
    #[error("expected something indexable")]
    ExpectedIndexable,
    #[error("expected a function")]
    ExpectedCallable,
    #[error("expected an enum")]
    ExpectedEnum,
    #[error("expected a struct")]
    ExpectedStruct,
    #[error("expected to be inside a loop")]
    ExpectedLoop,
    #[error("expected to be inside a moved closure")]
    ExpectedMovedClosure,
    #[error("expected to be inside a closure")]
    ExpectedClosure,
    #[error("expected something assignable")]
    ExpectedLeftExpression,

    #[error("unknown variable : {0}")]
    UnknownVar(ID),
    #[error("unknown type : {0}")]
    UnknownType(ID),
    #[error("unknown field")]
    UnknownField,

    #[error("cannot have double open range like -∞:+∞")]
    DoubleInfinitRange,

    #[error("incorrect function arguments")]
    IncorrectArguments,
    #[error("invalid arguments for struct {0}")]
    IncorrectStruct(String),
    #[error("invalid arguments for union {0}")]
    IncorrectUnion(String),
    #[error("invalid variant {0}")]
    IncorrectVariant(String),
    #[error("invalid pattern")]
    InvalidPattern,
    #[error("invalid range")]
    InvalidRange,

    #[error("incompatible types")]
    IncompatibleTypes,
    #[error("incompatible operation")]
    IncompatibleOperation,
    #[error("incompatible operands")]
    IncompatibleOperands,

    #[error("internal compilation error")]
    ConcurrencyError,
    #[error("unexpected error")]
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
pub enum Either {
    Static(Arc<StaticType>),
    User(Arc<UserType>),
}

pub type EType = Either;

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
    fn resolve<G: crate::GameEngineStaticFn>(
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
