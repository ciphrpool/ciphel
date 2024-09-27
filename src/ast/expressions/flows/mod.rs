pub mod flows_gencode;
pub mod flows_parse;
pub mod flows_resolve;
pub mod flows_typeof;

use std::fmt::Debug;

use crate::{
    ast::{
        statements::block::{BlockCommonApi, ClosureBlock, ExprBlock},
        types::Type,
        utils::strings::ID,
        TryParse,
    },
    semantic::{EType, Metadata, Resolve},
    vm::GenerateCode,
};

use super::{
    data::{Primitive, StrSlice},
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
    then_branch: ExprBlock,
    else_branch: ExprBlock,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    expr: Box<Expression>,
    cases: Cases<ExprBlock, ExprBlock>,
    else_branch: Option<ExprBlock>,
    metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionPattern {
    pub typename: ID,
    pub variant: ID,
    pub vars_names: Vec<ID>,
    pub vars_id: Option<Vec<u64>>,
    pub variant_value: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveCase<
    B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
> {
    pub patterns: Vec<Primitive>,
    pub block: B,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringCase<
    B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
> {
    pub patterns: Vec<StrSlice>,
    pub block: B,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCase<
    B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
> {
    pub patterns: Vec<(String, String, Option<u64>)>,
    pub block: B,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionCase<
    B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
> {
    pub pattern: UnionPattern,
    pub block: B,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cases<
    B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq, // IIFE
    C: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq, // CLOSURE
> {
    Primitive { cases: Vec<PrimitiveCase<B>> },
    String { cases: Vec<StringCase<B>> },
    Enum { cases: Vec<EnumCase<B>> },
    Union { cases: Vec<UnionCase<C>> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    try_branch: ExprBlock,
    else_branch: Option<ExprBlock>,
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
