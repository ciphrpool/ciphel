use nom::{combinator::map, multi::many0, sequence::delimited};

use crate::{
    ast::{
        statements::Statement,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError},
};

use super::Scope;

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
