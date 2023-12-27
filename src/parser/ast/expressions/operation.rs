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
            value(ComparaisonOperator::ELE, wst(lexem::ELE)),
            value(ComparaisonOperator::LE, wst(lexem::LE)),
            value(ComparaisonOperator::EGE, wst(lexem::EGE)),
            value(ComparaisonOperator::GE, wst(lexem::GE)),
            value(ComparaisonOperator::EQ, wst(lexem::EQ)),
            value(ComparaisonOperator::NEQ, wst(lexem::NEQ)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::expressions::data::{Data, Primitive};

    use super::*;

    #[test]
    fn valid_unary() {
        let res = UnaryOperation::parse("-10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Minus(Box::new(Expression::Data(Data::Primitive(
                Primitive::Number(10)
            )))),
            value
        );

        let res = UnaryOperation::parse("!true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Not(Box::new(Expression::Data(Data::Primitive(
                Primitive::Bool(true)
            )))),
            value
        );
    }

    #[test]
    fn valid_binary_math() {
        let res = BinaryOperation::parse("10 + 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Add,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 * 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Mult,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 / 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Div,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 % 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Mod,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(2))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 << 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Shl,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(2))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 >> 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Math(MathOperation {
                operator: MathOperator::Shr,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(2))))
            }),
            value
        );
    }

    #[test]
    fn valid_binary_logical() {
        let res = BinaryOperation::parse("true and true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Logical(LogicalOperation {
                operator: LogicalOperator::And,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
            }),
            value
        );

        let res = BinaryOperation::parse("true or true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Logical(LogicalOperation {
                operator: LogicalOperator::Or,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
            }),
            value
        );

        let res = BinaryOperation::parse("true xor true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Logical(LogicalOperation {
                operator: LogicalOperator::Xor,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
            }),
            value
        );

        let res = BinaryOperation::parse("true in true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Logical(LogicalOperation {
                operator: LogicalOperator::In,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
            }),
            value
        );
    }

    #[test]
    fn valid_binary_comparaison() {
        let res = BinaryOperation::parse("10 < 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::LE,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 <= 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::ELE,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 > 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::GE,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 >= 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::EGE,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 == 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::EQ,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );

        let res = BinaryOperation::parse("10 != 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            BinaryOperation::Comparaison(ComparaisonOperation {
                operator: ComparaisonOperator::NEQ,
                left: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                right: Box::new(Expression::Data(Data::Primitive(Primitive::Number(5))))
            }),
            value
        );
    }
}
