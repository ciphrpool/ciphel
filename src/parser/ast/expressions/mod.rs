use nom::{branch::alt, combinator::map, sequence::delimited};

use crate::parser::utils::{
    io::{PResult, Span},
    lexem,
    strings::wst,
};

use super::TryParse;

pub mod data;
pub mod error;
pub mod operation;
pub mod statements;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Data(data::Data),
    Operation(operation::Operation),
    Paren(Box<Expression>),
    Statement(statements::Statement),
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
        alt((
            map(
                delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                |value| Expression::Paren(Box::new(value)),
            ),
            map(data::Data::parse, |value| Expression::Data(value)),
            map(operation::Operation::parse, |value| {
                Expression::Operation(value)
            }),
            map(statements::Statement::parse, |value| {
                Expression::Statement(value)
            }),
            map(error::Error::parse, |value| Expression::Error(value)),
        ))(input)
    }
}
