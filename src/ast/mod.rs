use crate::ast::utils::io::{PResult, Span};

pub(crate) mod expressions;
pub(crate) mod statements;
pub(crate) mod types;
pub mod utils;

pub trait TryParse {
    fn parse(input: Span) -> PResult<Self>
    where
        Self: Sized;
}
