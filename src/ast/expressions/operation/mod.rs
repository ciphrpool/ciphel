use crate::{
    ast::{types::Type, utils::strings::ID},
    semantic::Metadata,
};

use super::{
    data::{CallArgs, Number},
    Atomic, CompletePath, Expression,
};
pub mod operation_gencode;
pub mod operation_parse;
pub mod operation_resolve;
pub mod operation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation {
    Minus {
        value: Box<Expression>,
        metadata: Metadata,
    },
    Not {
        value: Box<Expression>,
        metadata: Metadata,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccess {
    pub var: Box<Expression>,
    pub field: Box<Expression>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleAccess {
    pub var: Box<Expression>,
    pub index: usize,
    pub offset: Option<usize>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAccess {
    pub var: Box<Expression>,
    pub index: Box<Expression>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub var: Box<Expression>,
    pub args: CallArgs,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub lower: Box<Expression>,
    pub upper: Box<Expression>,
    pub incr: Option<Number>,
    pub inclusive: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Product {
    Mult {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    Div {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    Mod {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct Addition {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Substraction {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shift {
    Left {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    Right {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseAnd {
    left: Box<Expression>,
    right: Box<Expression>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseXOR {
    left: Box<Expression>,
    right: Box<Expression>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseOR {
    left: Box<Expression>,
    right: Box<Expression>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast {
    left: Box<Expression>,
    right: Type,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparaison {
    Less {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    LessEqual {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    Greater {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    GreaterEqual {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Equation {
    Equal {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
    NotEqual {
        left: Box<Expression>,
        right: Box<Expression>,
        metadata: Metadata,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalAnd {
    left: Box<Expression>,
    right: Box<Expression>,
    pub metadata: Metadata,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOr {
    left: Box<Expression>,
    right: Box<Expression>,
    pub metadata: Metadata,
}
