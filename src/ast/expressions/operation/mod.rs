use std::cell::Cell;

use crate::{
    ast::types::Type,
    semantic::{scope::ScopeApi, EType, Metadata},
};

use super::{data::Number, Atomic, Expression};
pub mod operation_gencode;
pub mod operation_parse;
pub mod operation_resolve;
pub mod operation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation<InnerScope: ScopeApi> {
    Minus {
        value: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Not {
        value: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range<InnerScope: ScopeApi> {
    pub lower: Box<Expression<InnerScope>>,
    pub upper: Box<Expression<InnerScope>>,
    pub incr: Option<Cell<Number>>,
    pub inclusive: bool,
    pub metadata: Metadata,
}

impl<Scope: ScopeApi> Range<Scope> {
    pub fn signature(&self) -> Option<EType> {
        self.metadata.signature()
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        Some(&self.metadata)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Product<InnerScope: ScopeApi> {
    Mult {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Div {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Mod {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct Addition<InnerScope: ScopeApi> {
    pub left: Box<Expression<InnerScope>>,
    pub right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Substraction<InnerScope: ScopeApi> {
    pub left: Box<Expression<InnerScope>>,
    pub right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shift<InnerScope: ScopeApi> {
    Left {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Right {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseAnd<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseXOR<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseOR<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Type,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparaison<InnerScope: ScopeApi> {
    Less {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    LessEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    Greater {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    GreaterEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Equation<InnerScope: ScopeApi> {
    Equal {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
    NotEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalAnd<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOr<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
    pub metadata: Metadata,
}
