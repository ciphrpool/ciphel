use crate::parser::{
    ast::TryParse,
    utils::io::{PResult, Span},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {}

impl TryParse for Scope {
    fn parse(input: Span) -> PResult<Self> {
        Ok((input, Scope {}))
    }
}
