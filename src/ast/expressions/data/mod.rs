use std::cell::Cell;

use crate::semantic::scope::scope::Scope;
use crate::{
    ast::{self, statements::declaration::TypedVar, utils::strings::ID},
    e_static, p_num,
    semantic::{
        scope::{
            static_types::{NumberType, PrimitiveType, StaticType},
            ClosureState,
        },
        EType, Either, Metadata, MutRc, SemanticError,
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
pub struct Variable {
    pub id: ID,
    pub from_field: Cell<bool>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(Cell<Number>),
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
    pub padding: Cell<usize>,
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
    pub scope: ExprScope,
    pub closed: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprScope {
    Scope(ast::statements::block::Block),
    Expr(ast::statements::block::Block),
}

impl ExprScope {
    pub fn scope(&self) -> Result<MutRc<Scope>, SemanticError> {
        match self {
            ExprScope::Scope(scope) => scope.scope(),
            ExprScope::Expr(scope) => scope.scope(),
        }
    }
    pub fn to_capturing(&self, state: ClosureState) {
        match self {
            ExprScope::Scope(value) => value.to_capturing(state),
            ExprScope::Expr(value) => value.to_capturing(state),
        }
    }
}
//     pub fn env_vars(&self) -> Result<&Vec<Rc<Var>>, SemanticError> {
//         match self {
//             ExprScope::Scope(block) => block.env_vars(),
//             ExprScope::Expr(block) => block.env_vars(),
//         }
//     }

//     // pub fn parameters_size(&self) -> Result<usize, SemanticError> {
//     //     match self {
//     //         ExprScope::Scope(value) => value.parameters_size(),
//     //         ExprScope::Expr(value) => value.parameters_size(),
//     //     }
//     // }

// }

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

    pub fn signature(&self) -> Option<EType> {
        match self {
            Data::Primitive(value) => match value {
                Primitive::Number(value) => match value.get() {
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
                Primitive::Bool(_) => Some(Either::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                )),
                Primitive::Char(_) => Some(Either::Static(
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
