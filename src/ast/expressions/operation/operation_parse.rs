use nom::{
    branch::alt,
    combinator::{map, opt, value},
    sequence::preceded,
};

use crate::{
    ast::{
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    semantic::scope::ScopeApi,
};

use super::{
    Atomic, BitwiseAnd, BitwiseOR, BitwiseXOR, Comparaison, Equation, Expression, HighOrdMath,
    Inclusion, LogicalAnd, LogicalOr, LowOrdMath, Shift, UnaryOperation,
};

impl<InnerScope: ScopeApi> TryParse for UnaryOperation<InnerScope> {
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
pub trait TryParseOperation<InnerScope: ScopeApi> {
    fn parse(input: Span) -> PResult<Expression<InnerScope>>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq)]
enum HighOrdMathOPERATOR {
    Mult,
    Div,
    Mod,
}
impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for HighOrdMath<InnerScope> {
    /*
     * @desc Parse Multiplication, division, modulo operation
     *
     * @grammar
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Atomic::parse(input)?;
        let (remainder, op) = opt(alt((
            value(HighOrdMathOPERATOR::Mult, wst(lexem::MULT)),
            value(HighOrdMathOPERATOR::Div, wst(lexem::DIV)),
            value(HighOrdMathOPERATOR::Mod, wst(lexem::MOD)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = Atomic::parse(remainder)?;
            let left = Box::new(Expression::Atomic(left));
            let right = Box::new(Expression::Atomic(right));
            Ok((
                remainder,
                Expression::HighOrdMath(match op {
                    HighOrdMathOPERATOR::Mult => HighOrdMath::Mult { left, right },
                    HighOrdMathOPERATOR::Div => HighOrdMath::Div { left, right },
                    HighOrdMathOPERATOR::Mod => HighOrdMath::Mod { left, right },
                }),
            ))
        } else {
            Ok((remainder, Expression::Atomic(left)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LowOrdMathOPERATOR {
    Add,
    Minus,
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for LowOrdMath<InnerScope> {
    /*
     * @desc Parse addition operation
     *
     * @grammar
     * LowM := HighM + HighM | HighM
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = HighOrdMath::parse(input)?;
        let (remainder, op) = opt(alt((
            value(LowOrdMathOPERATOR::Add, wst(lexem::ADD)),
            value(LowOrdMathOPERATOR::Minus, wst(lexem::MINUS)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = HighOrdMath::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::LowOrdMath(match op {
                    LowOrdMathOPERATOR::Add => LowOrdMath::Add { left, right },
                    LowOrdMathOPERATOR::Minus => LowOrdMath::Minus { left, right },
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ShiftOPERATOR {
    Left,
    Right,
}
impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Shift<InnerScope> {
    /*
     * @desc Parse bitwise shift operation
     *
     * @grammar
     * Shift := LowM (<<|>>) LowM | LowM
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = LowOrdMath::parse(input)?;
        let (remainder, op) = opt(alt((
            value(ShiftOPERATOR::Left, wst(lexem::SHL)),
            value(ShiftOPERATOR::Right, wst(lexem::SHR)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = LowOrdMath::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::Shift(match op {
                    ShiftOPERATOR::Left => Shift::Left { left, right },
                    ShiftOPERATOR::Right => Shift::Right { left, right },
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for BitwiseAnd<InnerScope> {
    /*
     * @desc Parse bitwise and operation
     *
     * @grammar
     * BAnd := Shift & Shift | Shift
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Shift::parse(input)?;
        let (remainder, op) = opt(wst(lexem::BAND))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = Shift::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::BitwiseAnd(BitwiseAnd { left, right }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for BitwiseXOR<InnerScope> {
    /*
     * @desc Parse bitwise xor operation
     *
     * @grammar
     * XOr := BAnd ^ BAnd | BAnd
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = BitwiseAnd::parse(input)?;
        let (remainder, op) = opt(wst(lexem::XOR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = BitwiseAnd::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::BitwiseXOR(BitwiseXOR { left, right }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for BitwiseOR<InnerScope> {
    /*
     * @desc Parse bitwise or operation
     *
     * @grammar
     * BOr := XOr \| XOr  | XOr
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = BitwiseXOR::parse(input)?;
        let (remainder, op) = opt(wst(lexem::BOR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = BitwiseXOR::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((remainder, Expression::BitwiseOR(BitwiseOR { left, right })))
        } else {
            Ok((remainder, left))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ComparaisonOPERATOR {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    In,
}
impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Comparaison<InnerScope> {
    /*
     * @desc Parse comparaison operation
     *
     * @grammar
     * CompOp := BOr (< |<= | >= | > | == | != | in ) BOr | BOr
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = BitwiseOR::parse(input)?;
        let (remainder, op) = opt(alt((
            value(ComparaisonOPERATOR::LessEqual, wst(lexem::ELE)),
            value(ComparaisonOPERATOR::Less, wst(lexem::LE)),
            value(ComparaisonOPERATOR::GreaterEqual, wst(lexem::EGE)),
            value(ComparaisonOPERATOR::Greater, wst(lexem::GE)),
            value(ComparaisonOPERATOR::Equal, wst(lexem::EQ)),
            value(ComparaisonOPERATOR::NotEqual, wst(lexem::NEQ)),
            value(ComparaisonOPERATOR::In, wst(lexem::IN)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = BitwiseOR::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                match op {
                    ComparaisonOPERATOR::Less => {
                        Expression::Comparaison(Comparaison::Less { left, right })
                    }
                    ComparaisonOPERATOR::LessEqual => {
                        Expression::Comparaison(Comparaison::LessEqual { left, right })
                    }
                    ComparaisonOPERATOR::Greater => {
                        Expression::Comparaison(Comparaison::Greater { left, right })
                    }
                    ComparaisonOPERATOR::GreaterEqual => {
                        Expression::Comparaison(Comparaison::GreaterEqual { left, right })
                    }
                    ComparaisonOPERATOR::Equal => {
                        Expression::Equation(Equation::Equal { left, right })
                    }
                    ComparaisonOPERATOR::NotEqual => {
                        Expression::Equation(Equation::NotEqual { left, right })
                    }
                    ComparaisonOPERATOR::In => Expression::Inclusion(Inclusion { left, right }),
                },
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for LogicalAnd<InnerScope> {
    /*
     * @desc Parse logical and operation
     *
     * @grammar
     * AndOp := CompOp And CompOp | CompOp
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Comparaison::<InnerScope>::parse(input)?;
        let (remainder, op) = opt(wst(lexem::AND))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = Comparaison::<InnerScope>::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::LogicalAnd(LogicalAnd { left, right }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for LogicalOr<InnerScope> {
    /*
     * @desc Parse logical or operation
     *
     * @grammar
     * Expr := AndOp Or AndOp | AndOp
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = LogicalAnd::<InnerScope>::parse(input)?;
        let (remainder, op) = opt(wst(lexem::OR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = LogicalAnd::<InnerScope>::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((remainder, Expression::LogicalOr(LogicalOr { left, right })))
        } else {
            Ok((remainder, left))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::expressions::data::{Data, Primitive},
        semantic::scope::scope_impl::MockScope,
    };

    use super::*;

    #[test]
    fn valid_unary() {
        let res = UnaryOperation::<MockScope>::parse("-10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Minus(Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                Primitive::Number(10)
            ))))),
            value
        );

        let res = UnaryOperation::<MockScope>::parse("!true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Not(Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                Primitive::Bool(true)
            ))))),
            value
        );
    }

    #[test]
    fn valid_binary_math() {
        let res = LowOrdMath::<MockScope>::parse("10 + 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LowOrdMath(LowOrdMath::Add {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                ))))
            }),
            value
        );

        let res = HighOrdMath::<MockScope>::parse("10 * 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::HighOrdMath(HighOrdMath::Mult {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                ))))
            }),
            value
        );

        let res = HighOrdMath::<MockScope>::parse("10 / 10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::HighOrdMath(HighOrdMath::Div {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                ))))
            }),
            value
        );

        let res = HighOrdMath::<MockScope>::parse("10 % 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::HighOrdMath(HighOrdMath::Mod {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(2)
                ))))
            }),
            value
        );

        let res = Shift::<MockScope>::parse("10 << 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Shift(Shift::Left {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(2)
                ))))
            }),
            value
        );

        let res = Shift::<MockScope>::parse("10 >> 2".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Shift(Shift::Right {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(2)
                ))))
            }),
            value
        );
    }

    #[test]
    fn valid_binary_logical() {
        let res = LogicalAnd::<MockScope>::parse("true and true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LogicalAnd(LogicalAnd {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                ))))
            }),
            value
        );

        let res = LogicalOr::<MockScope>::parse("true or true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LogicalOr(LogicalOr {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                ))))
            }),
            value
        );

        let res = BitwiseXOR::<MockScope>::parse("true ^ true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::BitwiseXOR(BitwiseXOR {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                ))))
            }),
            value
        );
    }

    #[test]
    fn valid_binary_comparaison() {
        let res = Comparaison::<MockScope>::parse("true in true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Inclusion(Inclusion {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 < 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::Less {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 <= 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::LessEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 > 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::Greater {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 >= 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::GreaterEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 == 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Equation(Equation::Equal {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 != 5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Equation(Equation::NotEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(5)
                ))))
            }),
            value
        );
    }
}
