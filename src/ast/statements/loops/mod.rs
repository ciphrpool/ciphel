use crate::ast::{
    expressions::{
        data::{Address, Slice, Tuple, Vector},
        Expression,
    },
    utils::strings::ID,
};

use super::{
    declaration::{DeclaredVar, PatternVar},
    scope::Scope,
};

pub mod loops_parse;
pub mod loops_resolve;
pub mod loops_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Loop {
    For(ForLoop),
    While(WhileLoop),
    Loop(Box<Scope>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    item: ForItem,
    iterator: ForIterator,
    scope: Box<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForItem {
    Id(ID),
    Pattern(PatternVar),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForIterator {
    Id(ID),
    Vec(Vector),
    Slice(Slice),
    Tuple(Tuple),
    Receive { addr: Address, timeout: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    condition: Box<Expression>,
    scope: Box<Scope>,
}
