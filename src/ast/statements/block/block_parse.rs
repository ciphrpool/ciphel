use std::sync::{Arc, RwLock};

use nom::{
    combinator::{cut, map},
    multi::many0,
    sequence::delimited,
};

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

use super::{Block, ClosureBlock, ExprBlock, FunctionBlock};

impl TryParse for Block {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| Block::new(value),
        )(input)
    }
}

impl TryParse for FunctionBlock {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| FunctionBlock::new(value),
        )(input)
    }
}

impl TryParse for ClosureBlock {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| ClosureBlock::new(value),
        )(input)
    }
}

impl TryParse for ExprBlock {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| ExprBlock::new(value),
        )(input)
    }
}
