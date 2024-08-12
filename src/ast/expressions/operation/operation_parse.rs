use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::digit1,
    combinator::{cut, map, opt, peek, value},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use nom_supreme::{error::ErrorTree, ParserExt};

use crate::{
    ast::{
        expressions::data::{Data, Variable},
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst},
        },
        TryParse,
    },
    semantic::Metadata,
};

use super::{
    Addition, Atomic, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, Expression,
    FieldAccess, FnCall, ListAccess, LogicalAnd, LogicalOr, Product, Range, Shift, Substraction,
    TupleAccess, UnaryOperation,
};

impl TryParse for UnaryOperation {
    /*
     * @desc Parse Unary opertion
     *
     * @grammar
     * Unary := - Expr | ! Expr
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                preceded(
                    wst(lexem::MINUS),
                    cut(Expression::parse).context("Invalid negation expression"),
                ),
                |value| UnaryOperation::Minus {
                    value: Box::new(value),
                    metadata: Metadata::default(),
                },
            ),
            map(
                preceded(
                    wst(lexem::NEGATION),
                    cut(Expression::parse).context("Invalid not expression"),
                ),
                |value| UnaryOperation::Not {
                    value: Box::new(value),
                    metadata: Metadata::default(),
                },
            ),
        ))(input)
    }
}

pub trait TryParseOperation {
    fn parse(input: Span) -> PResult<Expression>
    where
        Self: Sized;
}

impl TryParseOperation for TupleAccess {
    fn parse(input: Span) -> PResult<Expression>
    where
        Self: Sized,
    {
        let (remainder, left) = map(Atomic::parse, Expression::Atomic)(input)?;
        let (_, peeked_result) = opt(peek(preceded(
            wst(lexem::DOT),
            take_while_m_n(1, usize::MAX, |c: char| c.is_digit(10)),
        )))(remainder)?;

        if let Some(_) = peeked_result {
            let (remainder, (_, num)) = tuple((wst(lexem::DOT), digit1))(remainder)?;

            match num.parse::<usize>() {
                Ok(index) => {
                    return Ok((
                        remainder,
                        Expression::TupleAccess(TupleAccess {
                            var: Box::new(left),
                            index,
                            metadata: Metadata::default(),
                        }),
                    ));
                }
                Err(_) => {
                    return Err(nom::Err::Error(ErrorTree::Base {
                        location: remainder,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    }));
                }
            }
        } else {
            Ok((remainder, left))
        }
    }
}
impl TryParseOperation for ListAccess {
    fn parse(input: Span) -> PResult<Expression>
    where
        Self: Sized,
    {
        let (remainder, left) = TupleAccess::parse(input)?;
        let (remainder, index) = opt(delimited(
            wst(lexem::SQ_BRA_O),
            cut(Expression::parse.context("Invalid list accessing expression")),
            cut(wst(lexem::SQ_BRA_C)),
        ))(remainder)?;

        if let Some(index) = index {
            Ok((
                remainder,
                Expression::ListAccess(ListAccess {
                    var: Box::new(left),
                    index: Box::new(index),
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}
impl TryParseOperation for FieldAccess {
    fn parse(input: Span) -> PResult<Expression>
    where
        Self: Sized,
    {
        let (remainder, left) = ListAccess::parse(input)?;
        let (_, peeked) = opt(peek(wst(lexem::RANGE_SEP)))(remainder)?;
        if peeked.is_some() {
            return Ok((remainder, left));
        }
        let (remainder, dot_field) = opt(wst(lexem::DOT))(remainder)?;

        if let Some(_) = dot_field {
            let (remainder, num) = opt(digit1)(remainder)?;
            if let Some(try_num) = num {
                let index = try_num.parse::<usize>();
                if index.is_err() {
                    return Err(nom::Err::Error(ErrorTree::Base {
                        location: try_num,
                        kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
                    }));
                }
                let index = index.unwrap();
                return Ok((
                    remainder,
                    Expression::TupleAccess(TupleAccess {
                        var: Box::new(left),
                        index,
                        metadata: Metadata::default(),
                    }),
                ));
            }
            let (remainder, right) =
                cut(FieldAccess::parse.context("Invalid field accessing expression"))(remainder)?;
            Ok((
                remainder,
                Expression::FieldAccess(FieldAccess {
                    var: Box::new(left),
                    field: Box::new(right),
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, left))
        }
    }
}

impl FnCall {
    pub fn parse_statement(input: Span) -> PResult<Expression>
    where
        Self: Sized,
    {
        let (remainder, opt_libcall) = opt(tuple((
            parse_id,
            wst(lexem::SEP),
            parse_id,
            wst(lexem::PAR_O),
        )))(input)?;
        let (remainder, lib, fn_var, opt_params) = if let Some((lib, _, func, _)) = opt_libcall {
            let (remainder, opt_params) = opt(terminated(
                separated_list0(wst(lexem::COMA), Expression::parse),
                wst(lexem::PAR_C),
            ))(remainder)?;
            (
                remainder,
                Some(lib),
                Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: func,
                    metadata: Metadata::default(),
                    from_field: false,
                })))),
                opt_params,
            )
        } else {
            let (remainder, left) = FieldAccess::parse(input)?;
            let (remainder, opt_params) = opt(delimited(
                wst(lexem::PAR_O),
                separated_list0(wst(lexem::COMA), Expression::parse),
                wst(lexem::PAR_C),
            ))(remainder)?;
            (remainder, None, Box::new(left), opt_params)
        };
        if let Some(params) = opt_params {
            let left = Expression::FnCall(FnCall {
                lib,
                fn_var,
                params,
                metadata: Metadata::default(),
                platform: Default::default(),
                is_dynamic_fn: Default::default(),
            });
            Ok((remainder, left))
        } else {
            return Err(nom::Err::Error(ErrorTree::Base {
                location: remainder,
                kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
            }));
        }
    }
}
impl TryParseOperation for FnCall {
    fn parse(input: Span) -> PResult<Expression>
    where
        Self: Sized,
    {
        let (remainder, opt_libcall) = opt(tuple((
            parse_id,
            wst(lexem::SEP),
            parse_id,
            wst(lexem::PAR_O),
        )))(input)?;
        let (remainder, lib, fn_var, opt_params) = if let Some((lib, _, func, _)) = opt_libcall {
            let (remainder, opt_params) = cut(opt(terminated(
                separated_list0(wst(lexem::COMA), Expression::parse),
                wst(lexem::PAR_C),
            )))(remainder)?;
            (
                remainder,
                Some(lib),
                Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: func,
                    metadata: Metadata::default(),
                    from_field: false,
                })))),
                opt_params,
            )
        } else {
            let (remainder, left) = FieldAccess::parse(input)?;
            let (remainder, opt_params) = opt(delimited(
                wst(lexem::PAR_O),
                cut(separated_list0(wst(lexem::COMA), Expression::parse))
                    .context("Invalid function call expression"),
                cut(wst(lexem::PAR_C)),
            ))(remainder)?;
            (remainder, None, Box::new(left), opt_params)
        };
        if let Some(params) = opt_params {
            let (_, peeked) = opt(peek(wst(lexem::RANGE_SEP)))(remainder)?;
            let left = Expression::FnCall(FnCall {
                lib,
                fn_var,
                params,
                metadata: Metadata::default(),
                platform: Default::default(),
                is_dynamic_fn: Default::default(),
            });
            if peeked.is_some() {
                return Ok((remainder, left));
            }
            let (remainder, dot_field) = opt(wst(lexem::DOT))(remainder)?;

            if let Some(_) = dot_field {
                let (remainder, num) = opt(digit1)(remainder)?;
                if let Some(try_num) = num {
                    let index = try_num.parse::<usize>();
                    if index.is_err() {
                        return Err(nom::Err::Error(ErrorTree::Base {
                            location: try_num,
                            kind: nom_supreme::error::BaseErrorKind::Kind(
                                nom::error::ErrorKind::Fail,
                            ),
                        }));
                    }
                    let index = index.unwrap();
                    return Ok((
                        remainder,
                        Expression::TupleAccess(TupleAccess {
                            var: Box::new(left),
                            index,
                            metadata: Metadata::default(),
                        }),
                    ));
                }
                let (remainder, right) = FieldAccess::parse(remainder)?;
                Ok((
                    remainder,
                    Expression::FieldAccess(FieldAccess {
                        var: Box::new(left),
                        field: Box::new(right),
                        metadata: Metadata::default(),
                    }),
                ))
            } else {
                let (remainder, index) = opt(delimited(
                    wst(lexem::SQ_BRA_O),
                    Expression::parse,
                    wst(lexem::SQ_BRA_C),
                ))(remainder)?;

                if let Some(index) = index {
                    Ok((
                        remainder,
                        Expression::ListAccess(ListAccess {
                            var: Box::new(left),
                            index: Box::new(index),
                            metadata: Metadata::default(),
                        }),
                    ))
                } else {
                    Ok((remainder, left))
                }
            }
        } else {
            Ok((remainder, *fn_var))
        }
    }
}

impl TryParseOperation for Range {
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = LogicalOr::parse(input)?;
        let (remainder, op) = opt(pair(wst(lexem::RANGE_SEP), opt(wst(lexem::EQUAL))))(remainder)?;

        if let Some((_, inclusive)) = op {
            let (remainder, upper) =
                cut(LogicalOr::parse.context("Invalid range expression"))(remainder)?;
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

#[derive(Debug, Clone, PartialEq)]
enum HighOrdMathOPERATOR {
    Mult,
    Div,
    Mod,
}
impl TryParseOperation for Product {
    /*
     * @desc Parse Multiplication, division, modulo operation
     *
     * @grammar
     * HighM := Atom (* | / | % ) Atom | Atom
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = Cast::parse(input)?;
        let (remainder, op) = opt(alt((
            value(HighOrdMathOPERATOR::Mult, wst(lexem::MULT)),
            value(HighOrdMathOPERATOR::Div, wst(lexem::DIV)),
            value(HighOrdMathOPERATOR::Mod, wst(lexem::MOD)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) =
                cut(Product::parse.context("Invalid product (*,/,%) expression"))(remainder)?;
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

impl TryParseOperation for Addition {
    /*
     * @desc Parse addition operation
     *
     * @grammar
     * LowM := HighM + HighM | HighM
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = Product::parse(input)?;
        let (remainder, op) = opt(alt((
            value(LowOrdMathOPERATOR::Add, wst(lexem::ADD)),
            value(LowOrdMathOPERATOR::Minus, wst(lexem::MINUS)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) =
                cut(Addition::parse.context("Invalid addition (+,-) expression"))(remainder)?;
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
impl TryParseOperation for Shift {
    /*
     * @desc Parse bitwise shift operation
     *
     * @grammar
     * Shift := LowM (<<|>>) LowM | LowM
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = Addition::parse(input)?;
        let (remainder, op) = opt(alt((
            value(ShiftOPERATOR::Left, wst(lexem::SHL)),
            value(ShiftOPERATOR::Right, wst(lexem::SHR)),
        )))(remainder)?;

        if let Some(op) = op {
            let (remainder, right) =
                cut(Shift::parse.context("Invalid shift ( >> , << ) expression"))(remainder)?;
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

impl TryParseOperation for BitwiseAnd {
    /*
     * @desc Parse bitwise and operation
     *
     * @grammar
     * BAnd := Shift & Shift | Shift
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = Shift::parse(input)?;
        let (remainder, op) = opt(wst(lexem::BAND))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(BitwiseAnd::parse.context("Invalid bitwise and expression"))(remainder)?;
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

impl TryParseOperation for BitwiseXOR {
    /*
     * @desc Parse bitwise xor operation
     *
     * @grammar
     * XOr := BAnd ^ BAnd | BAnd
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = BitwiseAnd::parse(input)?;
        let (remainder, op) = opt(wst(lexem::XOR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(BitwiseXOR::parse.context("Invalid bitwise xor expression"))(remainder)?;
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

impl TryParseOperation for BitwiseOR {
    /*
     * @desc Parse bitwise or operation
     *
     * @grammar
     * BOr := XOr \| XOr  | XOr
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = BitwiseXOR::parse(input)?;
        let (remainder, op) = opt(wst(lexem::BOR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(BitwiseOR::parse.context("Invalid bitwise or expression"))(remainder)?;
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

impl TryParseOperation for Cast {
    /*
     * @desc Parse bitwise xor operation
     *
     * @grammar
     * Cast := Atom as Type | Atom
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = FnCall::parse(input)?;
        let (remainder, op) = opt(wst(lexem::AS))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(Type::parse.context("Invalid type casting expression"))(remainder)?;
            Ok((
                remainder,
                Expression::Cast(Cast {
                    left: left.into(),
                    right,
                    metadata: Metadata::default(),
                }),
            ))
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
}
impl TryParseOperation for Comparaison {
    /*
     * @desc Parse comparaison operation
     *
     * @grammar
     * CompOp := BOr (< |<= | >= | > | == | != | in ) BOr | BOr
     */
    fn parse(input: Span) -> PResult<Expression> {
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
            let (remainder, right) = cut(BitwiseOR::parse
                .context("Invalid comparaison ( == , != , <=, >=, <, >) expression"))(
                remainder
            )?;
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

impl TryParseOperation for LogicalAnd {
    /*
     * @desc Parse logical and operation
     *
     * @grammar
     * AndOp := CompOp And CompOp | CompOp
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = Comparaison::parse(input)?;
        let (remainder, op) = opt(wst(lexem::AND))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(LogicalAnd::parse.context("Invalid and expression"))(remainder)?;
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

impl TryParseOperation for LogicalOr {
    /*
     * @desc Parse logical or operation
     *
     * @grammar
     * Expr := AndOp Or AndOp | AndOp
     */
    fn parse(input: Span) -> PResult<Expression> {
        let (remainder, left) = LogicalAnd::parse(input)?;
        let (remainder, op) = opt(wst(lexem::OR))(remainder)?;

        if let Some(_op) = op {
            let (remainder, right) =
                cut(LogicalOr::parse.context("Invalid or expression"))(remainder)?;
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

    use crate::{
        ast::expressions::data::{Data, Number, Primitive},
        v_num,
    };

    use super::*;

    #[test]
    fn valid_unary() {
        let res = UnaryOperation::parse("-10".into());
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

        let res = UnaryOperation::parse("!true".into());
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
        let res = Addition::parse("10 + 10".into());
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

        let res = Addition::parse("10 - 10".into());
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

        let res = Product::parse("10 * 10".into());
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

        let res = Product::parse("10 / 10".into());
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

        let res = Product::parse("10 % 2".into());
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

        let res = Shift::parse("10 << 2".into());
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

        let res = Shift::parse("10 >> 2".into());
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
        let res = Addition::parse("10 + 10 + 10".into());
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

        let res = Product::parse("10 * 10 * 10".into());
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

        let res = LogicalAnd::parse("true and true and true".into());
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
        let res = LogicalAnd::parse("true and true".into());
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

        let res = LogicalOr::parse("true or true".into());
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

        let res = BitwiseXOR::parse("true ^ true".into());
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
        let res = Comparaison::parse("10 < 5".into());
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

        let res = Comparaison::parse("10 <= 5".into());
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

        let res = Comparaison::parse("10 > 5".into());
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

        let res = Comparaison::parse("10 >= 5".into());
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

        let res = Comparaison::parse("10 == 5".into());
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

        let res = Comparaison::parse("10 != 5".into());
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
        let res = Range::parse("1..10".into());
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

        let res = Range::parse("1..=10".into());
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

        let res = Range::parse("1u64..10u64".into());
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

    #[test]
    fn valid_fn_call() {
        let res = FnCall::parse("f(x,10)".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FnCall(FnCall {
                lib: None,
                fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "f".to_string().into(),
                    metadata: Metadata::default(),
                    from_field: false,
                })))),
                params: vec![
                    Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "x".to_string().into(),
                        metadata: Metadata::default(),
                        from_field: false,
                    }))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(10).into()
                    ))))
                ],
                metadata: Metadata::default(),
                platform: Default::default(),
                is_dynamic_fn: Default::default(),
            }),
            value
        );
    }

    #[test]
    fn valid_lib_call() {
        let res = FnCall::parse("core::f(x,10)".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FnCall(FnCall {
                lib: Some("core".to_string().into()),
                fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "f".to_string().into(),
                    metadata: Metadata::default(),
                    from_field: false,
                })))),
                params: vec![
                    Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "x".to_string().into(),
                        metadata: Metadata::default(),
                        from_field: false,
                    }))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(10).into()
                    ))))
                ],
                metadata: Metadata::default(),
                platform: Default::default(),
                is_dynamic_fn: Default::default(),
            }),
            value
        );
    }
}
