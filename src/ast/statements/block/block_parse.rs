use std::sync::{Arc, RwLock};

use nom::{
    branch::alt,
    combinator::{cut, map},
    multi::many0,
    sequence::delimited,
};

use crate::{
    ast::{
        expressions::Expression,
        statements::{self, return_stat::Return, Statement},
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    semantic::Metadata,
};

use super::{Block, ClosureBlock, ExprBlock, FunctionBlock, LambdaBlock};

impl TryParse for Block {
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, block) = map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| Block::new(value),
        )(input)?;

        let mut found_inline_return_statement = false;
        for statement in block.statements.iter() {
            if found_inline_return_statement {
                return Err(nom::Err::Failure(
                    nom_supreme::error::GenericErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    },
                ));
            }
            if let Statement::Return(Return::Inline { .. }) = statement {
                found_inline_return_statement = true;
            }
        }

        Ok((remainder, block))
    }
}

impl TryParse for FunctionBlock {
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, block) = map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| FunctionBlock::new(value),
        )(input)?;

        let mut found_inline_return_statement = false;
        for statement in block.statements.iter() {
            if found_inline_return_statement {
                return Err(nom::Err::Failure(
                    nom_supreme::error::GenericErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    },
                ));
            }
            if let Statement::Return(Return::Inline { .. }) = statement {
                found_inline_return_statement = true;
            }
        }

        Ok((remainder, block))
    }
}

impl TryParse for ClosureBlock {
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, block) = alt((
            map(
                delimited(
                    wst(lexem::BRA_O),
                    cut(many0(Statement::parse)),
                    cut(wst(lexem::BRA_C)),
                ),
                |value| ClosureBlock::new(value),
            ),
            map(Expression::parse, |expr| {
                let statement = Statement::Return(Return::Inline {
                    expr: Box::new(expr),
                    metadata: Metadata::default(),
                });
                ClosureBlock::new(vec![statement])
            }),
        ))(input)?;

        let mut found_inline_return_statement = false;
        for statement in block.statements.iter() {
            if found_inline_return_statement {
                return Err(nom::Err::Failure(
                    nom_supreme::error::GenericErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    },
                ));
            }
            if let Statement::Return(Return::Inline { .. }) = statement {
                found_inline_return_statement = true;
            }
        }

        Ok((remainder, block))
    }
}

impl TryParse for LambdaBlock {
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, block) = alt((
            map(
                delimited(
                    wst(lexem::BRA_O),
                    cut(many0(Statement::parse)),
                    cut(wst(lexem::BRA_C)),
                ),
                |value| LambdaBlock::new(value),
            ),
            map(Expression::parse, |expr| {
                let statement = Statement::Return(Return::Inline {
                    expr: Box::new(expr),
                    metadata: Metadata::default(),
                });
                LambdaBlock::new(vec![statement])
            }),
        ))(input)?;

        let mut found_inline_return_statement = false;
        for statement in block.statements.iter() {
            if found_inline_return_statement {
                return Err(nom::Err::Failure(
                    nom_supreme::error::GenericErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    },
                ));
            }
            if let Statement::Return(Return::Inline { .. }) = statement {
                found_inline_return_statement = true;
            }
        }

        Ok((remainder, block))
    }
}

impl TryParse for ExprBlock {
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, block) = map(
            delimited(
                wst(lexem::BRA_O),
                cut(many0(Statement::parse)),
                cut(wst(lexem::BRA_C)),
            ),
            |value| ExprBlock::new(value),
        )(input)?;

        let mut found_inline_return_statement = false;
        for statement in block.statements.iter() {
            if found_inline_return_statement {
                return Err(nom::Err::Failure(
                    nom_supreme::error::GenericErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    },
                ));
            }
            if let Statement::Return(Return::Inline { .. }) = statement {
                found_inline_return_statement = true;
            }
        }

        Ok((remainder, block))
    }
}
