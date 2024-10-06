use nom::{
    branch::alt,
    combinator::{cut, map},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};
use nom_supreme::ParserExt;

use crate::ast::{
    expressions::data::{Closure, Lambda},
    statements::assignation::AssignValue,
    types::{ClosureType, LambdaType, Type},
    utils::{
        error::squash,
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst, wst_closed},
    },
    TryParse,
};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl TryParse for Declaration {
    /*
     * @desc Parse Declaration
     *
     * @grammar
     * Declaration := let TypedVar ;
     *      | let DeclaredVar = AssignValue
     */
    fn parse(input: Span) -> PResult<Self> {
        preceded(
            wst_closed(lexem::LET),
            cut(alt((
                map(
                    terminated(
                        TypedVar::parse.context("Expected a valid variable definition"),
                        wst(lexem::SEMI_COLON),
                    ),
                    |value| Declaration::Declared(value),
                ),
                map(
                    separated_pair(DeclaredVar::parse, wst(lexem::EQUAL), AssignValue::parse),
                    |(left, right)| Declaration::Assigned { left, right },
                ),
                map(
                    separated_pair(
                        preceded(
                            wst_closed(lexem::REC),
                            separated_pair(
                                parse_id,
                                wst(lexem::COLON),
                                ClosureType::parse.context("Invalid type"),
                            ),
                        ),
                        wst(lexem::EQUAL),
                        terminated(Closure::parse, wst(lexem::SEMI_COLON)),
                    ),
                    |((name, signature), right)| Declaration::RecClosure {
                        id: None,
                        name,
                        signature,
                        right,
                    },
                ),
                map(
                    separated_pair(
                        preceded(
                            wst_closed(lexem::REC),
                            separated_pair(
                                parse_id,
                                wst(lexem::COLON),
                                LambdaType::parse.context("Invalid type"),
                            ),
                        ),
                        wst(lexem::EQUAL),
                        terminated(Lambda::parse, wst(lexem::SEMI_COLON)),
                    ),
                    |((name, signature), right)| Declaration::RecLambda {
                        id: None,
                        name,
                        signature,
                        right,
                    },
                ),
            ))),
        )(input)
    }
}

impl Declaration {
    /*
     * @desc Parse Declaration
     *
     * @grammar
     * Declaration := let DeclaredVar = AssignValue
     */
    pub fn parse_without_semicolon(input: Span) -> PResult<Self> {
        preceded(
            wst_closed(lexem::LET),
            cut(map(
                separated_pair(
                    DeclaredVar::parse,
                    wst(lexem::EQUAL),
                    AssignValue::parse_without_semicolon,
                ),
                |(left, right)| Declaration::Assigned { left, right },
            )),
        )(input)
    }
}

impl TryParse for TypedVar {
    /*
     * @desc Parse TypeVar
     *
     * @grammar
     * TypedVar := ID : Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                parse_id,
                wst(lexem::COLON),
                Type::parse.context("Invalid type"),
            ),
            |(name, signature)| TypedVar {
                id: None,
                signature,
                name,
            },
        )(input)
    }
}

impl TryParse for DeclaredVar {
    /*
     * @desc Parse Declared var
     *
     * @grammar
     * DeclaredVar := ID | TypedVar | PatternVar
     */
    fn parse(input: Span) -> PResult<Self> {
        squash(
            alt((
                map(TypedVar::parse, |value| DeclaredVar::Typed(value)),
                map(PatternVar::parse, |value| DeclaredVar::Pattern(value)),
                map(parse_id, |name| DeclaredVar::Id { name, id: None }),
            )),
            "Expected a valid variable or pattern",
        )(input)
    }
}

impl TryParse for PatternVar {
    /*
     * @desc Parse Pattern var
     *
     * @grammar
     * PatternVar :=
     *      | ID :: ID \( IDs \)
     *      | ID :: ID { IDs }
     *      | ID { IDs }
     *      | ID \( IDs \)
     *      | \(  IDs \)
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |(typename, vars)| PatternVar::StructFields {
                    typename,
                    vars,
                    ids: None,
                },
            ),
            map(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list1(wst(lexem::COMA), parse_id),
                    wst(lexem::PAR_C),
                ),
                |names| PatternVar::Tuple { names, ids: None },
            ),
        ))(input)
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Tuple},
                Atomic, Expression,
            },
            types::{NumberType, PrimitiveType},
        },
        semantic::Metadata,
        v_num,
    };

    use super::*;

    #[test]
    fn valid_declaration_declared() {
        let res = Declaration::parse("let x:u64;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Declared(TypedVar {
                id: None,
                signature: Type::Primitive(PrimitiveType::Number(NumberType::U64)),
                name: "x".to_string()
            }),
            value
        );
    }

    #[test]
    fn valid_declaration_assigned() {
        let res = Declaration::parse("let x:u64 = 10;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Typed(TypedVar {
                    name: "x".to_string(),
                    id: None,
                    signature: Type::Primitive(PrimitiveType::Number(NumberType::U64)),
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );

        let res = Declaration::parse("let x = 10;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Id {
                    name: "x".to_string().into(),
                    id: None
                },
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );

        let res = Declaration::parse("let (x,y) = (10,10);".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Pattern(PatternVar::Tuple {
                    names: vec!["x".to_string().into(), "y".to_string().into()],
                    ids: None
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(Data::Tuple(
                    Tuple {
                        value: vec![
                            Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                                Number::Unresolved(10).into()
                            )))),
                            Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                                Number::Unresolved(10).into()
                            ))))
                        ],
                        metadata: Metadata::default()
                    }
                )))))
            },
            value
        );
    }
}
