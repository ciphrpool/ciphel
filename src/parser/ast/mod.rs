use super::utils::io::{PResult, Span};

mod expressions;
mod statements;
mod types;

pub trait TryParse {
    fn parse(input: Span) -> PResult<Self>
    where
        Self: Sized;
}
