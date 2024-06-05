use std::{
    cell::{Cell, RefCell},
    sync::{Arc, RwLock},
};

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
    semantic::{scope::ClosureState, Metadata},
};

use super::Block;

impl TryParse for Block {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                many0(Statement::parse),
                wst(lexem::BRA_C),
            ),
            |value| Block {
                instructions: value,
                inner_scope: None,
                can_capture: Arc::new(RwLock::new(ClosureState::DEFAULT)),
                is_loop: Default::default(),

                caller: Default::default(),
                metadata: Metadata::default(),
            },
        )(input)
    }
}
