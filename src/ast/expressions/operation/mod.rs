use crate::{ast::types::Type, semantic::scope::ScopeApi};

use super::{Atomic, Expression};
pub mod operation_parse;
pub mod operation_resolve;
pub mod operation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation<InnerScope: ScopeApi> {
    Minus(Box<Expression<InnerScope>>),
    Not(Box<Expression<InnerScope>>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum HighOrdMath<InnerScope: ScopeApi> {
    Mult {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    Div {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    Mod {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum LowOrdMath<InnerScope: ScopeApi> {
    Add {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    Minus {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Shift<InnerScope: ScopeApi> {
    Left {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    Right {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseAnd<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseXOR<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseOR<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cast<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparaison<InnerScope: ScopeApi> {
    Less {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    LessEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    Greater {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    GreaterEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Equation<InnerScope: ScopeApi> {
    Equal {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
    NotEqual {
        left: Box<Expression<InnerScope>>,
        right: Box<Expression<InnerScope>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inclusion<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalAnd<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOr<InnerScope: ScopeApi> {
    left: Box<Expression<InnerScope>>,
    right: Box<Expression<InnerScope>>,
}
