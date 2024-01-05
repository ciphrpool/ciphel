pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;
use crate::ast::utils::strings::ID;

use super::{
    data::{Primitive, Variable},
    Expression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprFlow {
    If(IfExpr),
    Match(MatchExpr),
    Try(TryExpr),
    Call(FnCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    condition: Box<Expression>,
    main_branch: Box<Expression>,
    else_branch: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    expr: Box<Expression>,
    patterns: Vec<PatternExpr>,
    else_branch: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Primitive(Primitive),
    String(String),
    Enum {
        typename: ID,
        value: ID,
    },
    Union {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    Struct {
        typename: ID,
        vars: Vec<ID>,
    },
    Tuple(Vec<ID>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternExpr {
    pattern: Pattern,
    expr: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    try_branch: Box<Expression>,
    else_branch: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall {
    pub fn_var: Variable,
    pub params: Vec<Expression>,
}
