pub mod flows_gencode;
pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{types::Type, utils::strings::ID},
    semantic::{scope::ScopeApi, EType, Metadata},
    vm::platform::Lib,
};

use super::{
    data::{ExprScope, Primitive, StrSlice, Variable},
    Expression,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprFlow<InnerScope: ScopeApi> {
    If(IfExpr<InnerScope>),
    Match(MatchExpr<InnerScope>),
    Try(TryExpr<InnerScope>),
    Call(FnCall<InnerScope>),
    SizeOf(Type, Metadata),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr<InnerScope: ScopeApi> {
    condition: Box<Expression<InnerScope>>,
    then_branch: ExprScope<InnerScope>,
    else_branch: ExprScope<InnerScope>,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr<InnerScope: ScopeApi> {
    expr: Box<Expression<InnerScope>>,
    patterns: Vec<PatternExpr<InnerScope>>,
    else_branch: Option<ExprScope<InnerScope>>,
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
pub struct PatternExpr<InnerScope: ScopeApi> {
    pattern: Pattern,
    expr: ExprScope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr<InnerScope: ScopeApi> {
    try_branch: ExprScope<InnerScope>,
    else_branch: ExprScope<InnerScope>,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<InnerScope: ScopeApi> {
    pub fn_var: Variable<InnerScope>,
    pub params: Vec<Expression<InnerScope>>,
    pub metadata: Metadata,
    pub platform: Rc<RefCell<Option<Lib>>>,
}

impl<Scope: ScopeApi> ExprFlow<Scope> {
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            ExprFlow::If(IfExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Match(MatchExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Try(TryExpr { metadata, .. }) => Some(metadata),
            ExprFlow::Call(FnCall { metadata, .. }) => Some(metadata),
            ExprFlow::SizeOf(_, metadata) => Some(metadata),
        }
    }
    pub fn signature(&self) -> Option<EType> {
        match self {
            ExprFlow::If(IfExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::Match(MatchExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::Try(TryExpr { metadata, .. }) => metadata.signature(),
            ExprFlow::Call(FnCall { metadata, .. }) => metadata.signature(),
            ExprFlow::SizeOf(_, metadata) => metadata.signature(),
        }
    }
}
