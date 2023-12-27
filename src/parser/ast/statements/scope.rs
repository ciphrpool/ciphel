use nom::{combinator::map, multi::many0, sequence::delimited};

use crate::parser::{
    ast::TryParse,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
};

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub instructions: Vec<Statement>,
}

impl TryParse for Scope {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                many0(Statement::parse),
                wst(lexem::BRA_C),
            ),
            |value| Scope {
                instructions: value,
            },
        )(input)
    }
}
