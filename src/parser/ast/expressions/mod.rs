use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::fold_many0,
    sequence::delimited,
};

use crate::parser::{
    ast::expressions::operation::{LogicalOr, TryParseOperation},
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{eater, wst},
    },
};

use super::TryParse;

pub mod data;
pub mod error;
pub mod flows;
pub mod operation;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    HighOrdMath(operation::HighOrdMath),
    LowOrdMath(operation::LowOrdMath),
    Shift(operation::Shift),
    BitwiseAnd(operation::BitwiseAnd),
    BitwiseXOR(operation::BitwiseXOR),
    BitwiseOR(operation::BitwiseOR),
    Comparaison(operation::Comparaison),
    LogicalAnd(operation::LogicalAnd),
    LogicalOr(operation::LogicalOr),
    Atomic(Atomic),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atomic {
    Data(data::Data),
    UnaryOperation(operation::UnaryOperation),
    Paren(Box<Expression>),
    ExprFlow(flows::ExprFlow),
    Error(error::Error),
}

impl TryParse for Atomic {
    /*
     * @desc Parse an atomic expression
     *
     * @grammar
     * Atomic :=
     * | Data
     * | Negation
     * | \(  Expr \)
     * | ExprStatement
     * | Error
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                |value| Atomic::Paren(Box::new(value)),
            ),
            map(error::Error::parse, |value| Atomic::Error(value)),
            map(flows::ExprFlow::parse, |value| Atomic::ExprFlow(value)),
            map(data::Data::parse, |value| Atomic::Data(value)),
            map(operation::UnaryOperation::parse, |value| {
                Atomic::UnaryOperation(value)
            }),
        ))(input)
    }
}

impl TryParse for Expression {
    /*
     * @desc Parse an expression
     *
     * @grammar
     * Expr := AndOp Or AndOp | AndOp
     * AndOp := CompOp And CompOp | CompOp
     * CompOp := BOr (<|<=|>=|>|\=\=|\!\= | in ) BOr | BOr
     * BOr := XOr \| XOr  | XOr
     * XOr := BAnd ^ BAnd | BAnd
     * BAnd := Shift & Shift | Shift
     * Shift := LowM (<<|>>) LowM | LowM
     * LowM := HighM + HighM | HighM
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Self> {
        eater::ws(LogicalOr::parse)(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::expressions::{
        data::{Data, Primitive},
        operation::LowOrdMath,
    };

    use super::*;

    #[test]
    fn valid_expression() {
        let res = Expression::parse(" 1 + 2 * 4".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LowOrdMath(LowOrdMath::Add {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(1)
                )))),
                right: Box::new(Expression::HighOrdMath(operation::HighOrdMath::Mult {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(2)
                    )))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(4)
                    ))))
                }))
            }),
            value
        );
    }
}
