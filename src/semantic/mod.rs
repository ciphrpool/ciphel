use std::{
    collections::HashSet,
    sync::{Arc, Mutex, RwLock},
};

use crate::semantic::scope::scope::ScopeManager;
use crate::{ast::utils::strings::ID, vm::vm::Printer};

use self::scope::{static_types::StaticType, user_type_impl::UserType};

use scope::static_types::NumberType;
use thiserror::Error;
pub mod scope;

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
    #[error("expected a block")]
    ExpectedInlineBlock,
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

pub trait Desugar<O> {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<O>, SemanticError>;
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

impl CompatibleWith for EType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        match (self, other) {
            (EType::Static(x), EType::Static(y)) => x.compatible_with(y, scope_manager, scope_id),
            (
                EType::User {
                    id: id_x,
                    size: size_x,
                },
                EType::User {
                    id: id_y,
                    size: size_y,
                },
            ) => (*id_x == *id_y && *size_x == *size_y)
                .then(|| ())
                .ok_or(SemanticError::IncompatibleTypes),
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl SizeOf for EType {
    fn size_of(&self) -> usize {
        match self {
            EType::Static(value) => value.size_of(),
            EType::User { id, size } => *size,
        }
    }
}

impl MergeType for EType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError> {
        match (self, other) {
            (EType::Static(x), EType::Static(y)) => x.merge(y, scope_manager, scope_id),
            (
                EType::User {
                    id: id_x,
                    size: size_x,
                },
                EType::User {
                    id: id_y,
                    size: size_y,
                },
            ) => {
                if *id_x != *id_y || *size_x != *size_y {
                    return Err(SemanticError::IncompatibleTypes);
                }
                return Ok(EType::User {
                    id: *id_x,
                    size: *size_x,
                });
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

// impl Printer for EType {
//     fn build_printer(
//         &self,
//         instructions: &mut crate::vm::casm::CasmProgram,
//     ) -> Result<(), crate::vm::vm::CodeGenerationError> {
//         match self {
//             EType::Static(value) => value.build_printer(instructions),
//             EType::User { id, size } => todo!(),
//         }
//     }
// }
