use crate::ast::{
    expressions::{
        Expression,
    },
    utils::strings::ID,
};

use super::{block::Block, declaration::PatternVar};

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
pub struct ForLoop {
    item: ForItem,
    iterator: ForIterator,
    scope: Box<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForItem {
    Id(ID),
    Pattern(PatternVar),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForIterator {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    condition: Box<Expression>,
    scope: Box<Block>,
}
