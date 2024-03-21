use crate::{
    ast::expressions::{
        flows::{FnCall, Pattern},
        Expression,
    },
    semantic::scope::ScopeApi,
};

use super::scope::Scope;

pub mod flows_gencode;
pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Flow<InnerScope: ScopeApi> {
    If(IfStat<InnerScope>),
    Match(MatchStat<InnerScope>),
    Try(TryStat<InnerScope>),
    Call(CallStat<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStat<InnerScope: ScopeApi> {
    condition: Box<Expression<InnerScope>>,
    then_branch: Box<Scope<InnerScope>>,
    else_if_branches: Vec<(Expression<InnerScope>, Scope<InnerScope>)>,
    else_branch: Option<Box<Scope<InnerScope>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStat<InnerScope: ScopeApi> {
    expr: Box<Expression<InnerScope>>,
    patterns: Vec<PatternStat<InnerScope>>,
    else_branch: Option<Box<Scope<InnerScope>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternStat<InnerScope: ScopeApi> {
    pattern: Pattern,
    scope: Box<Scope<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStat<InnerScope: ScopeApi> {
    try_branch: Box<Scope<InnerScope>>,
    else_branch: Option<Box<Scope<InnerScope>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallStat<InnerScope: ScopeApi> {
    pub call: FnCall<InnerScope>,
}
