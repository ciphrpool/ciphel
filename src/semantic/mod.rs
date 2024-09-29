use std::{
    collections::HashSet,
    sync::{Arc, Mutex, RwLock},
};

use crate::semantic::scope::scope::ScopeManager;
use crate::{ast::utils::strings::ID, vm::CodeGenerationError};

use self::scope::{static_types::StaticType, user_types::UserType};

use scope::{
    static_types::NumberType,
    user_types::{Enum, Struct, Union},
};
use thiserror::Error;
pub mod scope;

#[derive(Debug, Clone, Error)]
pub enum SemanticError {
    #[error("NotResolvedYet")]
    NotResolvedYet,
    #[error("CoreAPIOverriding")]
    CoreAPIOverriding,

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
    #[error("expected to be inside a closure")]
    ExpectedClosure,
    #[error("expected to be inside a functio, a closure or a lambda")]
    ExpectedFunctionClosureLambda,
    #[error("expected something assignable")]
    ExpectedLeftExpression,

    #[error("unknown variable : {0}")]
    UnknownVar(String),
    #[error("unknown type : {0}")]
    UnknownType(String),
    #[error("unknown field")]
    UnknownField,

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
    fn resolve<E: crate::vm::external::Engine>(
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
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<O>, SemanticError>;
}

pub trait ResolveNumber {
    fn is_unresolved_number(&self) -> bool;
    fn resolve_number(&mut self, to: NumberType) -> Result<(), SemanticError>;
}

pub trait ResolveCore {
    fn resolve<E: crate::vm::external::Engine>(
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
    fn resolve_from_struct<E: crate::vm::external::Engine>(
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

impl EType {
    pub fn name(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<String, CodeGenerationError> {
        match self {
            EType::Static(static_type) => match static_type {
                StaticType::Primitive(primitive_type) => match primitive_type {
                    scope::static_types::PrimitiveType::Number(number_type) => match number_type {
                        NumberType::U8 => Ok("u8".to_string()),
                        NumberType::U16 => Ok("u16".to_string()),
                        NumberType::U32 => Ok("u32".to_string()),
                        NumberType::U64 => Ok("u64".to_string()),
                        NumberType::U128 => Ok("u128".to_string()),
                        NumberType::I8 => Ok("i8".to_string()),
                        NumberType::I16 => Ok("i16".to_string()),
                        NumberType::I32 => Ok("i32".to_string()),
                        NumberType::I64 => Ok("i64".to_string()),
                        NumberType::I128 => Ok("i128".to_string()),
                        NumberType::F64 => Ok("f64".to_string()),
                    },
                    scope::static_types::PrimitiveType::Char => Ok("char".to_string()),
                    scope::static_types::PrimitiveType::Bool => Ok("bool".to_string()),
                },
                StaticType::Slice(slice_type) => Ok(format!(
                    "[{0}]{1}",
                    slice_type.size,
                    slice_type.item_type.name(scope_manager, scope_id)?
                )),
                StaticType::String(string_type) => Ok("String".to_string()),
                StaticType::StrSlice(str_slice_type) => Ok("str".to_string()),
                StaticType::Vec(vec_type) => Ok(format!(
                    "Vec[{0}]",
                    vec_type.0.name(scope_manager, scope_id)?
                )),
                StaticType::Closure(closure_type) => Ok(format!(
                    "closed({0}) -> {1}",
                    closure_type
                        .params
                        .iter()
                        .filter_map(|p| p.name(scope_manager, scope_id).ok())
                        .collect::<Vec<String>>()
                        .join(","),
                    closure_type.ret.name(scope_manager, scope_id)?
                )),
                StaticType::Lambda(lambda_type) => Ok(format!(
                    "({0}) -> {1}",
                    lambda_type
                        .params
                        .iter()
                        .filter_map(|p| p.name(scope_manager, scope_id).ok())
                        .collect::<Vec<String>>()
                        .join(","),
                    lambda_type.ret.name(scope_manager, scope_id)?
                )),
                StaticType::Function(function_type) => Ok(format!(
                    "fn({0}) -> {1}",
                    function_type
                        .params
                        .iter()
                        .filter_map(|p| p.name(scope_manager, scope_id).ok())
                        .collect::<Vec<String>>()
                        .join(","),
                    function_type.ret.name(scope_manager, scope_id)?
                )),
                StaticType::Tuple(tuple_type) => Ok(format!(
                    "({0})",
                    tuple_type
                        .0
                        .iter()
                        .filter_map(|p| p.name(scope_manager, scope_id).ok())
                        .collect::<Vec<String>>()
                        .join(",")
                )),
                StaticType::Unit => Ok("Unit".to_string()),
                StaticType::Any => Ok("Any".to_string()),
                StaticType::Error => Ok("Error".to_string()),
                StaticType::Address(addr_type) => {
                    Ok(format!("&{0}", addr_type.0.name(scope_manager, scope_id)?))
                }
                StaticType::Map(map_type) => Ok(format!(
                    "Map[{0}]{1}",
                    map_type.keys_type.name(scope_manager, scope_id)?,
                    map_type.values_type.name(scope_manager, scope_id)?,
                )),
            },
            EType::User { id, size } => {
                let Ok(utype) = scope_manager.find_type_by_id(*id, scope_id) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                match utype {
                    UserType::Struct(Struct { id, fields }) => Ok(id),
                    UserType::Enum(Enum { id, values }) => Ok(id),
                    UserType::Union(Union { id, variants }) => Ok(id),
                }
            }
        }
    }
}
