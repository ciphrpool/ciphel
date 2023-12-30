use crate::ast::{
    expressions::{
        flows::{FnCall, Pattern},
        Expression,
    },
    utils::strings::ID,
};

use super::scope::Scope;

pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    If(IfStat),
    Match(MatchStat),
    Try(TryStat),
    Call(CallStat),
    Return(Return),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStat {
    condition: Box<Expression>,
    main_branch: Box<Scope>,
    else_branch: Option<Box<Scope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStat {
    expr: Box<Expression>,
    patterns: Vec<PatternStat>,
    else_branch: Option<Box<Scope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternStat {
    pattern: Pattern,
    scope: Box<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStat {
    try_branch: Box<Scope>,
    else_branch: Option<Box<Scope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallStat {
    pub call: FnCall,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Unit,
    Expr(Box<Expression>),
}
