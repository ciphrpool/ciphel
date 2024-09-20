use std::{
    collections::HashSet,
    sync::{Arc, Mutex, RwLock},
};

use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::ScopeManager;

use self::scope::{static_types::StaticType, user_type_impl::UserType};

use scope::static_types::NumberType;
use thiserror::Error;
pub mod scope;
pub mod utils;

#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    #[error("NotResolvedYet")]
    NotResolvedYet,
    #[error("PlatformAPIOverriding")]
    PlatformAPIOverriding,

    #[error("cannot infer type {0}")]
    CantInferType(String),

    #[error("some case are not covered {0:?}")]
    ExhaustiveCases(HashSet<String>),

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
    UnknownVar(String),
    #[error("unknown type : {0}")]
    UnknownType(String),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Info {
    Unresolved,
    Resolved {
        context: Option<EType>,
        signature: Option<EType>,
    },
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

impl Default for Metadata {
    fn default() -> Self {
        Self {
            info: Info::Unresolved,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EType {
    Static(StaticType),
    User { id: u64, size: usize },
}

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
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait ResolveNumber {
    fn is_unresolved_number(&self) -> bool;
    fn resolve_number(&mut self, to: NumberType) -> Result<(), SemanticError>;
}

pub trait ResolvePlatform {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<crate::ast::expressions::Expression>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized;
}

pub trait ResolveFromStruct {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError>
    where
        Self: Sized + Resolve;
}

pub trait CompatibleWith {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError>
    where
        Self: PartialEq,
    {
        (self == other)
            .then(|| ())
            .ok_or(SemanticError::IncompatibleTypes)
    }
}

pub trait TypeOf {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized;
}

pub trait MergeType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>;
}

pub trait SizeOf {
    fn size_of(&self) -> usize;
}
