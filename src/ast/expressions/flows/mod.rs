pub mod flows_gencode;
pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    ast::{types::Type, utils::strings::ID},
    semantic::{EType, Metadata},
    vm::platform::{stdlib::strings::StringsFn, Lib},
};

use super::{
    data::{ExprScope, Primitive, StrSlice, Variable},
    Expression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprFlow {
    If(IfExpr),
    Match(MatchExpr),
    Try(TryExpr),
    FCall(FCall),
    SizeOf(Type, Metadata),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    condition: Box<Expression>,
    then_branch: ExprScope,
    else_branch: ExprScope,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    expr: Box<Expression>,
    patterns: Vec<PatternExpr>,
    else_branch: Option<ExprScope>,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Primitive(Primitive),
    String(StrSlice),
    Enum {
        typename: ID,
        value: ID,
    },
    Union {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    // Struct {
    //     typename: ID,
    //     vars: Vec<ID>,
    // },
    // Tuple(Vec<ID>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternExpr {
    patterns: Vec<Pattern>,
    expr: ExprScope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    try_branch: ExprScope,
    else_branch: Option<ExprScope>,
    pop_last_err: bool,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatItem {
    Str(String),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FCall {
    pub value: Vec<FormatItem>,
    pub metadata: Metadata,
}

impl ExprFlow {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            ExprFlow::If(IfExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Match(MatchExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Try(TryExpr { metadata, .. }) => Some(metadata),
            ExprFlow::SizeOf(_, metadata) => Some(metadata),
            ExprFlow::FCall(FCall { metadata, .. }) => Some(metadata),
        }
    }
    pub fn metadata_mut(&mut self) -> Option<&mut Metadata> {
        match self {
            ExprFlow::If(IfExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Match(MatchExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Try(TryExpr { metadata, .. }) => Some(metadata),
            ExprFlow::SizeOf(_, metadata) => Some(metadata),
            ExprFlow::FCall(FCall { metadata, .. }) => Some(metadata),
        }
    }
    pub fn signature(&self) -> Option<EType> {
        match self {
            ExprFlow::If(IfExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::Match(MatchExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::Try(TryExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::FCall(FCall { metadata, .. }) => metadata.signature(),
            ExprFlow::SizeOf(_, metadata) => metadata.signature(),
        }
    }
}
