use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{self, utils::strings::ID},
    semantic::{
        scope::{
            chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
            var_impl::Var, ScopeApi,
        },
        Metadata, SemanticError,
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
    Vec(Vector<InnerScope>),
    Closure(Closure<InnerScope>),
    Chan(Channel<InnerScope>),
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
    Number(Number),
    Float(f64),
    Bool(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slice<InnerScope: ScopeApi> {
    String {
        value: String,
        metadata: Metadata,
    },
    List {
        value: MultiData<InnerScope>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vector<InnerScope: ScopeApi> {
    Init {
        value: MultiData<InnerScope>,
        metadata: Metadata,
    },
    Def {
        capacity: usize,
        metadata: Metadata,
    },
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
    env: Rc<RefCell<HashMap<ID, Rc<Var>>>>,
    scope: ExprScope<InnerScope>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprScope<InnerScope: ScopeApi> {
    Scope(ast::statements::scope::Scope<InnerScope>),
    Expr(ast::statements::scope::Scope<InnerScope>),
}

impl<InnerScope: ScopeApi> ExprScope<InnerScope> {
    pub fn find_outer_vars(&self) -> Result<HashMap<ID, Rc<Var>>, SemanticError> {
        match self {
            ExprScope::Scope(scope) => scope.find_outer_vars(),
            ExprScope::Expr(scope) => scope.find_outer_vars(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureParam {
    Full { id: ID, signature: ast::types::Type },
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
pub enum Channel<InnerScope: ScopeApi> {
    Receive {
        addr: Address<InnerScope>,
        timeout: usize,
        metadata: Metadata,
    },
    Send {
        addr: Address<InnerScope>,
        msg: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Init {
        value: String,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct<InnerScope: ScopeApi> {
    id: ID,
    fields: Vec<(String, Expression<InnerScope>)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union<InnerScope: ScopeApi> {
    typename: ID,
    variant: ID,
    fields: Vec<(ID, Expression<InnerScope>)>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    typename: ID,
    value: ID,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Map<InnerScope: ScopeApi> {
    Init {
        fields: Vec<(KeyData<InnerScope>, Expression<InnerScope>)>,
        metadata: Metadata,
    },
    Def {
        length: usize,
        capacity: usize,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyData<InnerScope: ScopeApi> {
    Primitive(Primitive),
    Slice(Slice<InnerScope>),
    Address(Address<InnerScope>),
    Enum(Enum),
}
