use crate::ast::utils::io::{PResult, Span};

pub mod expressions;
pub mod modules;
pub(crate) mod statements;
pub(crate) mod types;
pub mod utils;

pub trait TryParse {
    fn parse(input: Span) -> PResult<Self>
    where
        Self: Sized;
}
