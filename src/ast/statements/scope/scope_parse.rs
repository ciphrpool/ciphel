use std::cell::{Cell, RefCell};

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
    semantic::{
        scope::{ClosureState, ScopeApi},
        Metadata,
    },
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
                can_capture: Cell::new(ClosureState::DEFAULT),
                is_loop: Cell::new(false),
                is_generator: Cell::new(false),
                caller: Default::default(),
                metadata: Metadata::default(),
            },
        )(input)
    }
}
