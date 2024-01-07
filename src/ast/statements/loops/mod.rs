use crate::{
    ast::{
        expressions::{
            data::{Address, Slice, Tuple, Vector},
            Expression,
        },
        utils::strings::ID,
    },
    semantic::scope::ScopeApi,
};

use super::{declaration::PatternVar, scope::Scope};

pub mod loops_parse;
pub mod loops_resolve;
pub mod loops_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Loop<InnerScope: ScopeApi> {
    For(ForLoop<InnerScope>),
    While(WhileLoop<InnerScope>),
    Loop(Box<Scope<InnerScope>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop<InnerScope: ScopeApi> {
    item: ForItem,
    iterator: ForIterator<InnerScope>,
    scope: Box<Scope<InnerScope>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForItem {
    Id(ID),
    Pattern(PatternVar),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForIterator<InnerScope: ScopeApi> {
    Id(ID),
    Vec(Vector<InnerScope>),
    Slice(Slice<InnerScope>),
    Receive { addr: Address, timeout: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop<InnerScope: ScopeApi> {
    condition: Box<Expression<InnerScope>>,
    scope: Box<Scope<InnerScope>>,
}
