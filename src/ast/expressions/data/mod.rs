use crate::ast::statements::block::{Block, ClosureBlock, ExprBlock};
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::SizeOf;
use crate::{
    ast::{self, statements::declaration::TypedVar, utils::strings::ID},
    e_static, p_num,
    semantic::{
        scope::{
            static_types::{PrimitiveType, StaticType},
            ClosureState,
        },
        EType, Metadata, SemanticError,
    },
};

use super::{Atomic, Expression};

pub mod data_gencode;
pub mod data_parse;
pub mod data_resolve;
pub mod data_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Primitive(Primitive),
    Slice(Slice),
    StrSlice(StrSlice),
    Vec(Vector),
    Closure(Closure),
    Tuple(Tuple),
    Address(Address),
    PtrAccess(PtrAccess),
    Variable(Variable),
    Unit,
    Map(Map),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableState {
    Variable { id: u64 },
    Field { offset: usize, size: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub state: Option<VariableState>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(Number),
    Bool(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F64(f64),
    Unresolved(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub value: MultiData,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrSlice {
    pub value: String,
    pub padding: usize,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub value: MultiData,
    pub metadata: Metadata,
    pub length: usize,
    pub capacity: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub value: MultiData,
    pub metadata: Metadata,
}

pub type MultiData = Vec<Expression>;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    params: Vec<ClosureParam>,
    pub scope: ClosureBlock,
    pub closed: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureParam {
    Full(TypedVar),
    Minimal(ID),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    pub value: Box<Atomic>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrAccess {
    pub value: Box<Atomic>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub id: ID,
    pub fields: Vec<(ID, Expression)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub typename: ID,
    pub variant: ID,
    pub fields: Vec<(ID, Expression)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub typename: ID,
    pub value: ID,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub fields: Vec<(Expression, Expression)>,
    pub metadata: Metadata,
}

impl Data {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            Data::Primitive(_) => None,
            Data::Slice(Slice { value: _, metadata }) => Some(metadata),
            Data::Vec(Vector {
                value: _,
                metadata,
                length: _,
                capacity: _,
            }) => Some(metadata),
            Data::Closure(Closure { metadata, .. }) => Some(metadata),
            Data::Tuple(Tuple { value: _, metadata }) => Some(metadata),
            Data::Address(Address { value: _, metadata }) => Some(metadata),
            Data::PtrAccess(PtrAccess { value: _, metadata }) => Some(metadata),
            Data::Variable(Variable { metadata, .. }) => Some(metadata),
            Data::Unit => None,
            Data::Map(Map {
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Struct(Struct {
                id: _,
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Union(Union {
                typename: _,
                variant: _,
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Enum(Enum {
                typename: _,
                value: _,
                metadata,
            }) => Some(metadata),
            Data::StrSlice(StrSlice { metadata, .. }) => Some(metadata),
        }
    }

    pub fn metadata_mut(&mut self) -> Option<&mut Metadata> {
        match self {
            Data::Primitive(_) => None,
            Data::Slice(Slice { value: _, metadata }) => Some(metadata),
            Data::Vec(Vector {
                value: _,
                metadata,
                length: _,
                capacity: _,
            }) => Some(metadata),
            Data::Closure(Closure { metadata, .. }) => Some(metadata),
            Data::Tuple(Tuple { value: _, metadata }) => Some(metadata),
            Data::Address(Address { value: _, metadata }) => Some(metadata),
            Data::PtrAccess(PtrAccess { value: _, metadata }) => Some(metadata),
            Data::Variable(Variable { metadata, .. }) => Some(metadata),
            Data::Unit => None,
            Data::Map(Map {
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Struct(Struct {
                id: _,
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Union(Union {
                typename: _,
                variant: _,
                fields: _,
                metadata,
            }) => Some(metadata),
            Data::Enum(Enum {
                typename: _,
                value: _,
                metadata,
            }) => Some(metadata),
            Data::StrSlice(StrSlice { metadata, .. }) => Some(metadata),
        }
    }

    pub fn signature(&self) -> Option<EType> {
        match self {
            Data::Primitive(value) => match value {
                Primitive::Number(value) => match value {
                    Number::U8(_) => Some(p_num!(U8)),
                    Number::U16(_) => Some(p_num!(U16)),
                    Number::U32(_) => Some(p_num!(U32)),
                    Number::U64(_) => Some(p_num!(U64)),
                    Number::U128(_) => Some(p_num!(U128)),
                    Number::I8(_) => Some(p_num!(I8)),
                    Number::I16(_) => Some(p_num!(I16)),
                    Number::I32(_) => Some(p_num!(I32)),
                    Number::I64(_) => Some(p_num!(I64)),
                    Number::I128(_) => Some(p_num!(I128)),
                    Number::F64(_) => Some(p_num!(F64)),
                    Number::Unresolved(_) => None,
                },
                Primitive::Bool(_) => Some(EType::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                )),
                Primitive::Char(_) => Some(EType::Static(
                    StaticType::Primitive(PrimitiveType::Char).into(),
                )),
            },
            Data::Slice(Slice { value: _, metadata }) => metadata.signature(),
            Data::Vec(Vector {
                value: _,
                metadata,
                length: _,
                capacity: _,
            }) => metadata.signature(),
            Data::Closure(Closure { metadata, .. }) => metadata.signature(),
            Data::Tuple(Tuple { value: _, metadata }) => metadata.signature(),
            Data::Address(Address { value: _, metadata }) => metadata.signature(),
            Data::PtrAccess(PtrAccess { value: _, metadata }) => metadata.signature(),
            Data::Variable(Variable { metadata, .. }) => metadata.signature(),
            Data::Unit => Some(e_static!(StaticType::Unit)),
            Data::Map(Map {
                fields: _,
                metadata,
            }) => metadata.signature(),
            Data::Struct(Struct {
                id: _,
                fields: _,
                metadata,
            }) => metadata.signature(),
            Data::Union(Union {
                typename: _,
                variant: _,
                fields: _,
                metadata,
            }) => metadata.signature(),
            Data::Enum(Enum {
                typename: _,
                value: _,
                metadata,
            }) => metadata.signature(),
            Data::StrSlice(StrSlice { metadata, .. }) => metadata.signature(),
        }
    }
}

impl Primitive {
    pub fn size_of(&self) -> usize {
        match self {
            Primitive::Number(Number::U8(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::U8,
                )
                .size_of()
            }
            Primitive::Number(Number::U16(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::U16,
                )
                .size_of()
            }
            Primitive::Number(Number::U32(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::U32,
                )
                .size_of()
            }
            Primitive::Number(Number::U64(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::U64,
                )
                .size_of()
            }
            Primitive::Number(Number::U128(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::U128,
                )
                .size_of()
            }
            Primitive::Number(Number::I8(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I8,
                )
                .size_of()
            }
            Primitive::Number(Number::I16(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I16,
                )
                .size_of()
            }
            Primitive::Number(Number::I32(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I32,
                )
                .size_of()
            }
            Primitive::Number(Number::I64(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I64,
                )
                .size_of()
            }
            Primitive::Number(Number::I128(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I128,
                )
                .size_of()
            }
            Primitive::Number(Number::F64(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::F64,
                )
                .size_of()
            }
            Primitive::Number(Number::Unresolved(_)) => {
                crate::semantic::scope::static_types::PrimitiveType::Number(
                    crate::semantic::scope::static_types::NumberType::I64,
                )
                .size_of()
            }
            Primitive::Bool(_) => {
                crate::semantic::scope::static_types::PrimitiveType::Bool.size_of()
            }
            Primitive::Char(_) => {
                crate::semantic::scope::static_types::PrimitiveType::Char.size_of()
            }
        }
    }
}
