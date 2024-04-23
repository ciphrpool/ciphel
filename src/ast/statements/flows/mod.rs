use crate::ast::expressions::{
    flows::{FnCall, Pattern},
    Expression,
};

use super::block::Block;

pub mod flows_gencode;
pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    If(IfStat),
    Match(MatchStat),
    Try(TryStat),
    Call(CallStat),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStat {
    condition: Box<Expression>,
    then_branch: Box<Block>,
    else_if_branches: Vec<(Expression, Block)>,
    else_branch: Option<Box<Block>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStat {
    expr: Box<Expression>,
    patterns: Vec<PatternStat>,
    else_branch: Option<Box<Block>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternStat {
    pattern: Pattern,
    scope: Box<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStat {
    try_branch: Box<Block>,
    else_branch: Option<Box<Block>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallStat {
    pub call: FnCall,
}
