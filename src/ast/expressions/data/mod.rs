use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::{
        self,
        statements::{
            declaration::TypedVar,
            loops::{ForItem, ForIterator},
        },
        utils::strings::ID,
    },
    semantic::{
        scope::{
            static_types::{NumberType, PrimitiveType, StaticType},
            var_impl::Var,
            ClosureState, ScopeApi,
        },
        AccessLevel, EType, Either, Metadata, MutRc, SemanticError,
    },
};

use super::Expression;

pub mod data_gencode;
pub mod data_parse;
pub mod data_resolve;
pub mod data_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Data<InnerScope: ScopeApi> {
    Primitive(Primitive),
    Slice(Slice<InnerScope>),
    StrSlice(StrSlice),
    Vec(Vector<InnerScope>),
    Closure(Closure<InnerScope>),
    Tuple(Tuple<InnerScope>),
    Address(Address<InnerScope>),
    PtrAccess(PtrAccess<InnerScope>),
    Variable(Variable<InnerScope>),
    Unit,
    Map(Map<InnerScope>),
    Struct(Struct<InnerScope>),
    Union(Union<InnerScope>),
    Enum(Enum),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable<InnerScope: ScopeApi> {
    Var(VarID),
    FieldAccess(FieldAccess<InnerScope>),
    NumAccess(NumAccess<InnerScope>),
    ListAccess(ListAccess<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarID {
    pub id: ID,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAccess<InnerScope: ScopeApi> {
    pub var: Box<Variable<InnerScope>>,
    pub index: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccess<InnerScope: ScopeApi> {
    pub var: Box<Variable<InnerScope>>,
    pub field: Box<Variable<InnerScope>>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumAccess<InnerScope: ScopeApi> {
    pub var: Box<Variable<InnerScope>>,
    pub index: usize,
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
pub struct Slice<InnerScope: ScopeApi> {
    pub value: MultiData<InnerScope>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrSlice {
    pub value: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector<InnerScope: ScopeApi> {
    pub value: MultiData<InnerScope>,
    pub metadata: Metadata,
    pub length: usize,
    pub capacity: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple<InnerScope: ScopeApi> {
    pub value: MultiData<InnerScope>,
    pub metadata: Metadata,
}

pub type MultiData<InnerScope> = Vec<Expression<InnerScope>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure<InnerScope: ScopeApi> {
    params: Vec<ClosureParam>,
    pub scope: ExprScope<InnerScope>,
    pub closed: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprScope<InnerScope: ScopeApi> {
    Scope(ast::statements::scope::Scope<InnerScope>),
    Expr(ast::statements::scope::Scope<InnerScope>),
}

impl<InnerScope: ScopeApi> ExprScope<InnerScope> {
    pub fn scope(&self) -> Result<MutRc<InnerScope>, SemanticError> {
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
//             ExprScope::Scope(scope) => scope.env_vars(),
//             ExprScope::Expr(scope) => scope.env_vars(),
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
pub struct Address<InnerScope: ScopeApi> {
    pub value: Variable<InnerScope>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PtrAccess<InnerScope: ScopeApi> {
    pub value: Variable<InnerScope>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct<InnerScope: ScopeApi> {
    pub id: ID,
    pub fields: Vec<(String, Expression<InnerScope>)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union<InnerScope: ScopeApi> {
    pub typename: ID,
    pub variant: ID,
    pub fields: Vec<(ID, Expression<InnerScope>)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub typename: ID,
    pub value: ID,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map<InnerScope: ScopeApi> {
    pub fields: Vec<(KeyData<InnerScope>, Expression<InnerScope>)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyData<InnerScope: ScopeApi> {
    Primitive(Primitive),
    StrSlice(StrSlice),
    Address(Address<InnerScope>),
    Enum(Enum),
}

impl<Scope: ScopeApi> Data<Scope> {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            Data::Primitive(_) => None,
            Data::Slice(Slice { value, metadata }) => Some(metadata),
            Data::Vec(Vector {
                value,
                metadata,
                length,
                capacity,
            }) => Some(metadata),
            Data::Closure(Closure { metadata, .. }) => Some(metadata),
            Data::Tuple(Tuple { value, metadata }) => Some(metadata),
            Data::Address(Address { value, metadata }) => Some(metadata),
            Data::PtrAccess(PtrAccess { value, metadata }) => Some(metadata),
            Data::Variable(v) => v.metadata(),
            Data::Unit => None,
            Data::Map(Map { fields, metadata }) => Some(metadata),
            Data::Struct(Struct {
                id,
                fields,
                metadata,
            }) => Some(metadata),
            Data::Union(Union {
                typename,
                variant,
                fields,
                metadata,
            }) => Some(metadata),
            Data::Enum(Enum {
                typename,
                value,
                metadata,
            }) => Some(metadata),
            Data::StrSlice(StrSlice { value, metadata }) => Some(metadata),
        }
    }

    pub fn signature(&self) -> Option<EType> {
        match self {
            Data::Primitive(value) => match value {
                Primitive::Number(value) => match value.get() {
                    Number::U8(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U8)).into(),
                    )),
                    Number::U16(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U16)).into(),
                    )),
                    Number::U32(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U32)).into(),
                    )),
                    Number::U64(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    )),
                    Number::U128(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U128)).into(),
                    )),
                    Number::I8(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I8)).into(),
                    )),
                    Number::I16(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I16)).into(),
                    )),
                    Number::I32(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I32)).into(),
                    )),
                    Number::I64(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    )),
                    Number::I128(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I128)).into(),
                    )),
                    Number::F64(_) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into(),
                    )),
                    Number::Unresolved(_) => None,
                },
                Primitive::Bool(_) => Some(Either::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                )),
                Primitive::Char(_) => Some(Either::Static(
                    StaticType::Primitive(PrimitiveType::Char).into(),
                )),
            },
            Data::Slice(Slice { value, metadata }) => metadata.signature(),
            Data::Vec(Vector {
                value,
                metadata,
                length,
                capacity,
            }) => metadata.signature(),
            Data::Closure(Closure { metadata, .. }) => metadata.signature(),
            Data::Tuple(Tuple { value, metadata }) => metadata.signature(),
            Data::Address(Address { value, metadata }) => metadata.signature(),
            Data::PtrAccess(PtrAccess { value, metadata }) => metadata.signature(),
            Data::Variable(v) => v.signature(),
            Data::Unit => Some(Either::Static(StaticType::Unit.into())),
            Data::Map(Map { fields, metadata }) => metadata.signature(),
            Data::Struct(Struct {
                id,
                fields,
                metadata,
            }) => metadata.signature(),
            Data::Union(Union {
                typename,
                variant,
                fields,
                metadata,
            }) => metadata.signature(),
            Data::Enum(Enum {
                typename,
                value,
                metadata,
            }) => metadata.signature(),
            Data::StrSlice(StrSlice { value, metadata }) => metadata.signature(),
        }
    }
}
