pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;
use crate::{ast::utils::strings::ID, semantic::scope::ScopeApi};

use super::{
    data::{ExprScope, Primitive, Variable},
    Expression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprFlow<InnerScope: ScopeApi> {
    If(IfExpr<InnerScope>),
    Match(MatchExpr<InnerScope>),
    Try(TryExpr<InnerScope>),
    Call(FnCall<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr<InnerScope: ScopeApi> {
    condition: Box<Expression<InnerScope>>,
    main_branch: ExprScope<InnerScope>,
    else_branch: ExprScope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr<InnerScope: ScopeApi> {
    expr: Box<Expression<InnerScope>>,
    patterns: Vec<PatternExpr<InnerScope>>,
    else_branch: ExprScope<InnerScope>,
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
pub struct PatternExpr<InnerScope: ScopeApi> {
    pattern: Pattern,
    expr: ExprScope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr<InnerScope: ScopeApi> {
    try_branch: ExprScope<InnerScope>,
    else_branch: ExprScope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<InnerScope: ScopeApi> {
    pub fn_var: Variable<InnerScope>,
    pub params: Vec<Expression<InnerScope>>,
}
