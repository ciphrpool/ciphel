use nom::{
    branch::alt,
    combinator::{map, opt, value},
    sequence::{delimited, pair, preceded, tuple},
};

use crate::{
    ast::{
        expressions::data::ExprScope,
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{eater, parse_id, wst},
        },
        TryParse,
    },
    semantic::{scope::ScopeApi, Metadata},
};

use super::{
    Addition, Atomic, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, Expression,
    LogicalAnd, LogicalOr, Product, Range, Shift, Substraction, UnaryOperation,
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
                UnaryOperation::Minus {
                    value: Box::new(value),
                    metadata: Metadata::default(),
                }
            }),
            map(preceded(wst(lexem::NEGATION), Expression::parse), |value| {
                UnaryOperation::Not {
                    value: Box::new(value),
                    metadata: Metadata::default(),
                }
            }),
        ))(input)
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Range<InnerScope> {
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = LogicalOr::parse(input)?;
        let (remainder, op) = opt(pair(wst(lexem::RANGE_SEP), opt(wst(lexem::EQUAL))))(remainder)?;

        if let Some((_, inclusive)) = op {
            let (remainder, upper) = LogicalOr::parse(remainder)?;
            let lower = Box::new(left);
            let upper = Box::new(upper);
            Ok((
                remainder,
                Expression::Range(Range {
                    lower,
                    upper,
                    incr: None,
                    inclusive: inclusive.is_some(),
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, left))
        }
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
impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Product<InnerScope> {
    /*
     * @desc Parse Multiplication, division, modulo operation
     *
     * @grammar
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Cast::parse(input)?;
        let (remainder, op) = opt(alt((
            value(HighOrdMathOPERATOR::Mult, wst(lexem::MULT)),
            value(HighOrdMathOPERATOR::Div, wst(lexem::DIV)),
            value(HighOrdMathOPERATOR::Mod, wst(lexem::MOD)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = Product::<InnerScope>::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::Product(match op {
                    HighOrdMathOPERATOR::Mult => Product::Mult {
                        left,
                        right,
                        metadata: Metadata::default(),
                    },
                    HighOrdMathOPERATOR::Div => Product::Div {
                        left,
                        right,
                        metadata: Metadata::default(),
                    },
                    HighOrdMathOPERATOR::Mod => Product::Mod {
                        left,
                        right,
                        metadata: Metadata::default(),
                    },
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LowOrdMathOPERATOR {
    Add,
    Minus,
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Addition<InnerScope> {
    /*
     * @desc Parse addition operation
     *
     * @grammar
     * LowM := HighM + HighM | HighM
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Product::parse(input)?;
        let (remainder, op) = opt(alt((
            value(LowOrdMathOPERATOR::Add, wst(lexem::ADD)),
            value(LowOrdMathOPERATOR::Minus, wst(lexem::MINUS)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = Addition::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                match op {
                    LowOrdMathOPERATOR::Add => Expression::Addition(Addition {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
                    LowOrdMathOPERATOR::Minus => Expression::Substraction(Substraction {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
                },
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
        let (remainder, left) = Addition::parse(input)?;
        let (remainder, op) = opt(alt((
            value(ShiftOPERATOR::Left, wst(lexem::SHL)),
            value(ShiftOPERATOR::Right, wst(lexem::SHR)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = Shift::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::Shift(match op {
                    ShiftOPERATOR::Left => Shift::Left {
                        left,
                        right,
                        metadata: Metadata::default(),
                    },
                    ShiftOPERATOR::Right => Shift::Right {
                        left,
                        right,
                        metadata: Metadata::default(),
                    },
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
            let (remainder, right) = BitwiseAnd::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::BitwiseAnd(BitwiseAnd {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
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
            let (remainder, right) = BitwiseXOR::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::BitwiseXOR(BitwiseXOR {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
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
            let (remainder, right) = BitwiseOR::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::BitwiseOR(BitwiseOR {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl<InnerScope: ScopeApi> TryParseOperation<InnerScope> for Cast<InnerScope> {
    /*
     * @desc Parse bitwise xor operation
     *
     * @grammar
     * Cast := Atom as Type | Atom
     */
    fn parse(input: Span) -> PResult<Expression<InnerScope>> {
        let (remainder, left) = Atomic::parse(input)?;
        let (remainder, op) = opt(wst(lexem::AS))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) = Type::parse(remainder)?;
            let left = Box::new(Expression::Atomic(left));
            Ok((
                remainder,
                Expression::Cast(Cast {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, Expression::Atomic(left)))
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
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) = BitwiseOR::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                match op {
                    ComparaisonOPERATOR::Less => Expression::Comparaison(Comparaison::Less {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
                    ComparaisonOPERATOR::LessEqual => {
                        Expression::Comparaison(Comparaison::LessEqual {
                            left,
                            right,
                            metadata: Metadata::default(),
                        })
                    }
                    ComparaisonOPERATOR::Greater => Expression::Comparaison(Comparaison::Greater {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
                    ComparaisonOPERATOR::GreaterEqual => {
                        Expression::Comparaison(Comparaison::GreaterEqual {
                            left,
                            right,
                            metadata: Metadata::default(),
                        })
                    }
                    ComparaisonOPERATOR::Equal => Expression::Equation(Equation::Equal {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
                    ComparaisonOPERATOR::NotEqual => Expression::Equation(Equation::NotEqual {
                        left,
                        right,
                        metadata: Metadata::default(),
                    }),
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
            let (remainder, right) = LogicalAnd::<InnerScope>::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::LogicalAnd(LogicalAnd {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
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
            let (remainder, right) = LogicalOr::<InnerScope>::parse(remainder)?;
            let left = Box::new(left);
            let right = Box::new(right);
            Ok((
                remainder,
                Expression::LogicalOr(LogicalOr {
                    left,
                    right,
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::expressions::data::{Data, Number, Primitive},
        semantic::scope::scope_impl::MockScope,
        v_num,
    };

    use super::*;

    #[test]
    fn valid_unary() {
        let res = UnaryOperation::<MockScope>::parse("-10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Minus {
                value: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                metadata: Metadata::default()
            },
            value
        );

        let res = UnaryOperation::<MockScope>::parse("!true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            UnaryOperation::Not {
                value: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_binary_math() {
        let res = Addition::<MockScope>::parse("10 + 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Addition(Addition {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                metadata: Metadata::default(),
            }),
            value
        );

        let res = Addition::<MockScope>::parse("10 - 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Substraction(Substraction {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Product::<MockScope>::parse("10 * 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Product(Product::Mult {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Product::<MockScope>::parse("10 / 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Product(Product::Div {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Product::<MockScope>::parse("10 % 2".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Product(Product::Mod {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 2
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Shift::<MockScope>::parse("10 << 2".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Shift(Shift::Left {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 2
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Shift::<MockScope>::parse("10 >> 2".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Shift(Shift::Right {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 2
                ))))),
                metadata: Metadata::default()
            }),
            value
        );
    }

    #[test]
    fn valid_multiple_binary() {
        let res = Addition::<MockScope>::parse("10 + 10 + 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Addition(Addition {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Addition(Addition {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 10
                    ))))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 10
                    ))))),
                    metadata: Metadata::default(),
                })),
                metadata: Metadata::default(),
            }),
            value
        );

        let res = Product::<MockScope>::parse("10 * 10 * 10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Product(Product::Mult {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Product(Product::Mult {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 10
                    ))))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 10
                    ))))),
                    metadata: Metadata::default(),
                })),
                metadata: Metadata::default(),
            }),
            value
        );

        let res = LogicalAnd::<MockScope>::parse("true and true and true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LogicalAnd(LogicalAnd {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::LogicalAnd(LogicalAnd {
                    left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Bool(true)
                    )))),
                    right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Bool(true)
                    )))),
                    metadata: Metadata::default(),
                })),
                metadata: Metadata::default(),
            }),
            value
        );
    }

    #[test]
    fn valid_binary_logical() {
        let res = LogicalAnd::<MockScope>::parse("true and true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LogicalAnd(LogicalAnd {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = LogicalOr::<MockScope>::parse("true or true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::LogicalOr(LogicalOr {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = BitwiseXOR::<MockScope>::parse("true ^ true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::BitwiseXOR(BitwiseXOR {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                metadata: Metadata::default()
            }),
            value
        );
    }

    #[test]
    fn valid_binary_comparaison() {
        let res = Comparaison::<MockScope>::parse("10 < 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::Less {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 <= 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::LessEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 > 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::Greater {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 >= 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Comparaison(Comparaison::GreaterEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 == 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Equation(Equation::Equal {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Comparaison::<MockScope>::parse("10 != 5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Equation(Equation::NotEqual {
                left: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 10
                ))))),
                right: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 5
                ))))),
                metadata: Metadata::default()
            }),
            value
        );
    }

    #[test]
    fn valid_range() {
        let res = Range::<MockScope>::parse("1..10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Range(Range {
                lower: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::Unresolved(1).into())
                )))),
                upper: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::Unresolved(10).into())
                )))),
                incr: None,
                inclusive: false,
                metadata: Metadata::default()
            }),
            value
        );

        let res = Range::<MockScope>::parse("1..=10".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Range(Range {
                lower: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::Unresolved(1).into())
                )))),
                upper: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::Unresolved(10).into())
                )))),
                incr: None,
                inclusive: true,
                metadata: Metadata::default()
            }),
            value
        );

        let res = Range::<MockScope>::parse("1u64..10u64".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Range(Range {
                lower: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::U64(1).into())
                )))),
                upper: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Number::U64(10).into())
                )))),
                incr: None,
                inclusive: false,
                metadata: Metadata::default()
            }),
            value
        );
    }
}
