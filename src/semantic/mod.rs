use std::sync::{Arc, Mutex, RwLock};

use crate::semantic::scope::scope::Scope;
use crate::{ast::utils::strings::ID};

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

    #[error("CantReturn")]
    CantReturn,
    #[error("CantInferType")]
    CantInferType,
    #[error("CantRegisterType")]
    CantRegisterType,
    #[error("CantRegisterVar")]
    CantRegisterVar,

    #[error("ExpectedBoolean")]
    ExpectedBoolean,
    #[error("ExpectedIterable")]
    ExpectedIterable,
    #[error("ExpectedIndexable")]
    ExpectedIndexable,
    #[error("ExpectedCallable")]
    ExpectedCallable,
    #[error("ExpectedEnum")]
    ExpectedEnum,
    #[error("ExpectedStruct")]
    ExpectedStruct,
    #[error("ExpectedLoop")]
    ExpectedLoop,
    #[error("ExpectedMovedClosure")]
    ExpectedMovedClosure,
    #[error("ExpectedClosure")]
    ExpectedClosure,
    #[error("ExpectedLeftExpression")]
    ExpectedLeftExpression,

    #[error("UnknownVar")]
    UnknownVar(ID),
    #[error("UnknownType")]
    UnknownType(ID),
    #[error("UnknownField")]
    UnknownField,

    #[error("DoubleInfinitRange")]
    DoubleInfinitRange,

    #[error("IncorrectArguments")]
    IncorrectArguments,
    #[error("IncorrectStruct")]
    IncorrectStruct,
    #[error("IncorrectVariant")]
    IncorrectVariant,
    #[error("InvalidPattern")]
    InvalidPattern,
    #[error("InvalidRange")]
    InvalidRange,

    #[error("IncompatibleTypes")]
    IncompatibleTypes,
    #[error("IncompatibleOperation")]
    IncompatibleOperation,
    #[error("IncompatibleOperands")]
    IncompatibleOperands,

    #[error("ConcurrencyError")]
    ConcurrencyError,
    #[error("Default")]
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
