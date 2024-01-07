use std::cell::RefCell;

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
    semantic::{scope::ScopeApi},
};

use super::Scope;

impl<Inner: ScopeApi> TryParse for Scope<Inner> {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                many0(Statement::parse),
                wst(lexem::BRA_C),
            ),
            |value| Scope {
                instructions: value,
                inner_scope: RefCell::new(None),
            },
        )(input)
    }
}
