use nom::{
    branch::alt,
    combinator::{cut, map},
    sequence::delimited,
};

use crate::parser::utils::{
    io::{PResult, Span},
    lexem,
    strings::wst,
};

use super::TryParse;

pub mod data;
pub mod error;
pub mod flows;
pub mod operation;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Data(data::Data),
    Operation(operation::Operation),
    Paren(Box<Expression>),
    ExprFlow(flows::ExprFlow),
    Error(error::Error),
}

impl TryParse for Expression {
    /*
     * @desc Parse an expression
     *
     * @grammar
     * Expr :=
     * | Data
     * | Negation
     * | BinaryOperation
     * | \(  Expr \)
     * | ExprStatement
     * | Error
     */
    fn parse(input: Span) -> PResult<Self> {
        cut(alt((
            map(
                delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                |value| Expression::Paren(Box::new(value)),
            ),
            map(data::Data::parse, |value| Expression::Data(value)),
            map(operation::Operation::parse, |value| {
                Expression::Operation(value)
            }),
            map(flows::ExprFlow::parse, |value| Expression::ExprFlow(value)),
            map(error::Error::parse, |value| Expression::Error(value)),
        )))(input)
    }
}
