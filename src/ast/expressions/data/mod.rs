use crate::{
    ast::{
        self,
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            numbers::{parse_float, parse_number, Number},
            strings::{
                eater, parse_id,
                string_parser::{parse_char, parse_string},
                wst, ID,
            },
        },
        TryParse,
    },
    semantic::{EitherType, Resolve, ScopeApi, SemanticError},
};

use super::Expression;

pub mod data_parse;
pub mod data_resolve;
pub mod data_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Primitive(Primitive),
    Slice(Slice),
    Vec(Vector),
    Closure(Closure),
    Chan(Channel),
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
pub enum Variable {
    Var(VarID),
    FieldAccess(FieldAccess),
    ListAccess(ListAccess),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarID(pub ID);

#[derive(Debug, Clone, PartialEq)]
pub struct ListAccess {
    var: Box<Variable>,
    index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccess {
    pub var: Box<Variable>,
    pub field: Box<Variable>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(Number),
    Float(f64),
    Bool(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slice {
    String(String),
    List(MultiData),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vector {
    Init(MultiData),
    Def { length: usize, capacity: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple(pub MultiData);

pub type MultiData = Vec<Expression>;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    params: Vec<ClosureParam>,
    scope: ClosureScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureScope {
    Scope(ast::statements::scope::Scope),
    Expr(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureParam {
    Full { id: ID, signature: ast::types::Type },
    Minimal(ID),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address(pub Box<Expression>);

#[derive(Debug, Clone, PartialEq)]
pub struct PtrAccess(pub Box<Expression>);

#[derive(Debug, Clone, PartialEq)]
pub enum Channel {
    Receive { addr: Address, timeout: usize },
    Send { addr: Address, msg: Box<Expression> },
    Init(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Struct {
    Inline {
        id: ID,
        data: MultiData,
    },
    Field {
        id: ID,
        fields: Vec<(String, Expression)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Union {
    Inline {
        typename: ID,
        variant: ID,
        data: MultiData,
    },
    Field {
        typename: ID,
        variant: ID,
        fields: Vec<(String, Expression)>,
    },
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Enum {
    typename: ID,
    value: ID,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Map {
    Init { fields: Vec<(KeyData, Expression)> },
    Def { length: usize, capacity: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyData {
    Number(Number),
    Bool(bool),
    Char(char),
    String(String),
    Address(Address),
    Enum(Enum),
}
