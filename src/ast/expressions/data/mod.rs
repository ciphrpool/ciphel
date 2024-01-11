use crate::{
    ast::{
        self,
        utils::{numbers::Number, strings::ID},
    },
    semantic::scope::ScopeApi,
};

use super::Expression;

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
pub struct VarID(pub ID);

#[derive(Debug, Clone, PartialEq)]
pub struct ListAccess<InnerScope: ScopeApi> {
    var: Box<Variable<InnerScope>>,
    index: Box<Expression<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccess<InnerScope: ScopeApi> {
    pub var: Box<Variable<InnerScope>>,
    pub field: Box<Variable<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumAccess<InnerScope: ScopeApi> {
    pub var: Box<Variable<InnerScope>>,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(Number),
    Float(f64),
    Bool(bool),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slice<InnerScope: ScopeApi> {
    String(String),
    List(MultiData<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vector<InnerScope: ScopeApi> {
    Init(MultiData<InnerScope>),
    Def { length: usize, capacity: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple<InnerScope: ScopeApi>(pub MultiData<InnerScope>);

pub type MultiData<InnerScope> = Vec<Expression<InnerScope>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Closure<InnerScope: ScopeApi> {
    params: Vec<ClosureParam>,
    scope: ExprScope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprScope<InnerScope: ScopeApi> {
    Scope(ast::statements::scope::Scope<InnerScope>),
    Expr(ast::statements::scope::Scope<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureParam {
    Full { id: ID, signature: ast::types::Type },
    Minimal(ID),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Address<InnerScope: ScopeApi>(pub Variable<InnerScope>);

#[derive(Debug, Clone, PartialEq)]
pub struct PtrAccess<InnerScope: ScopeApi>(pub Variable<InnerScope>);

#[derive(Debug, Clone, PartialEq)]
pub enum Channel<InnerScope: ScopeApi> {
    Receive {
        addr: Address<InnerScope>,
        timeout: usize,
    },
    Send {
        addr: Address<InnerScope>,
        msg: Box<Expression<InnerScope>>,
    },
    Init(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct<InnerScope: ScopeApi> {
    id: ID,
    fields: Vec<(String, Expression<InnerScope>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union<InnerScope: ScopeApi> {
    typename: ID,
    variant: ID,
    fields: Vec<(ID, Expression<InnerScope>)>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Enum {
    typename: ID,
    value: ID,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Map<InnerScope: ScopeApi> {
    Init {
        fields: Vec<(KeyData<InnerScope>, Expression<InnerScope>)>,
    },
    Def {
        length: usize,
        capacity: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyData<InnerScope: ScopeApi> {
    Primitive(Primitive),
    Slice(Slice<InnerScope>),
    Address(Address<InnerScope>),
    Enum(Enum),
}
