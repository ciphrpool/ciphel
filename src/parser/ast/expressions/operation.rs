use nom::{
    branch::alt,
    combinator::{map, value},
    sequence::{preceded, separated_pair, tuple},
};

use crate::parser::{
    ast::TryParse,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
};

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Unary(UnaryOperation),
    Binary(BinaryOperation),
}

impl TryParse for Operation {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(UnaryOperation::parse, |value| Operation::Unary(value)),
            map(BinaryOperation::parse, |value| Operation::Binary(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperation {
    Minus(Box<Expression>),
    Not(Box<Expression>),
}

impl TryParse for UnaryOperation {
    /*
     * @desc Parse Unary opertion
     *
     * @grammar
     * Unary := - Expr | ! Expr
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(preceded(wst(lexem::MINUS), Expression::parse), |value| {
                UnaryOperation::Minus(Box::new(value))
            }),
            map(preceded(wst(lexem::NEGATION), Expression::parse), |value| {
                UnaryOperation::Not(Box::new(value))
            }),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperation {
    Math(MathOperation),
    Logical(LogicalOperation),
    Comparaison(ComparaisonOperation),
}

impl TryParse for BinaryOperation {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(MathOperation::parse, |value| BinaryOperation::Math(value)),
            map(LogicalOperation::parse, |value| {
                BinaryOperation::Logical(value)
            }),
            map(ComparaisonOperation::parse, |value| {
                BinaryOperation::Comparaison(value)
            }),
        ))(input)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct MathOperation {
    operator: MathOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

impl TryParse for MathOperation {
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((Expression::parse, MathOperator::parse, Expression::parse)),
            |(left, operator, right)| MathOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MathOperator {
    Add,
    Mult,
    Div,
    Mod,
    Shl,
    Shr,
}

impl TryParse for MathOperator {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(MathOperator::Add, wst(lexem::ADD)),
            value(MathOperator::Mult, wst(lexem::MULT)),
            value(MathOperator::Div, wst(lexem::DIV)),
            value(MathOperator::Mod, wst(lexem::MOD)),
            value(MathOperator::Shl, wst(lexem::SHL)),
            value(MathOperator::Shr, wst(lexem::SHR)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalOperation {
    operator: LogicalOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

impl TryParse for LogicalOperation {
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((Expression::parse, LogicalOperator::parse, Expression::parse)),
            |(left, operator, right)| LogicalOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    Or,
    And,
    Xor,
    In,
}

impl TryParse for LogicalOperator {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(LogicalOperator::Or, wst(lexem::OR)),
            value(LogicalOperator::And, wst(lexem::AND)),
            value(LogicalOperator::Xor, wst(lexem::XOR)),
            value(LogicalOperator::In, wst(lexem::IN)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComparaisonOperation {
    operator: ComparaisonOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

impl TryParse for ComparaisonOperation {
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                Expression::parse,
                ComparaisonOperator::parse,
                Expression::parse,
            )),
            |(left, operator, right)| ComparaisonOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparaisonOperator {
    LE,
    ELE,
    GE,
    EGE,
    EQ,
    NEQ,
}
impl TryParse for ComparaisonOperator {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(ComparaisonOperator::LE, wst(lexem::LE)),
            value(ComparaisonOperator::ELE, wst(lexem::ELE)),
            value(ComparaisonOperator::GE, wst(lexem::GE)),
            value(ComparaisonOperator::EGE, wst(lexem::EGE)),
            value(ComparaisonOperator::EQ, wst(lexem::EQ)),
            value(ComparaisonOperator::NEQ, wst(lexem::NEQ)),
        ))(input)
    }
}
