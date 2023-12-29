use crate::{
    ast::{
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError},
};

use super::{Atomic, Expression};
pub mod operation_parse;
pub mod operation_resolve;
pub mod operation_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation {
    Minus(Box<Expression>),
    Not(Box<Expression>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum HighOrdMath {
    Mult {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Div {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Mod {
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum LowOrdMath {
    Add {
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Shift {
    Left {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Right {
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseAnd {
    left: Box<Expression>,
    right: Box<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseXOR {
    left: Box<Expression>,
    right: Box<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BitwiseOR {
    left: Box<Expression>,
    right: Box<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Comparaison {
    Less {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    LessEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Greater {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    GreaterEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Equal {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    NotEqual {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    In {
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalAnd {
    left: Box<Expression>,
    right: Box<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOr {
    left: Box<Expression>,
    right: Box<Expression>,
}
