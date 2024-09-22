use crate::ast::{expressions::Expression, utils::strings::ID};

use super::{block::Block, declaration::PatternVar, Statement};

pub mod loops_gencode;
pub mod loops_parse;
pub mod loops_resolve;
pub mod loops_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Loop {
    For(ForLoop),
    While(WhileLoop),
    Loop(Box<Block>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Assignation(super::assignation::Assignation),
    Declaration(super::declaration::Declaration),
}

pub type ForInits = Vec<ForInit>;
pub type ForCondition = Option<Expression>;
pub type ForIncrements = Vec<super::assignation::Assignation>;

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    indices: ForInits,
    condition: ForCondition,
    increments: ForIncrements,
    block: Box<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    condition: Box<Expression>,
    block: Box<Block>,
}
