use nom::{
    branch::alt,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::{
    ast::{
        statements::assignation::AssignValue,
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst},
        },
        TryParse,
    },
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
            wst(lexem::LET),
            alt((
                map(
                    terminated(TypedVar::parse, wst(lexem::SEMI_COLON)),
                    |value| Declaration::Declared(value),
                ),
                map(
                    separated_pair(DeclaredVar::parse, wst(lexem::EQUAL), AssignValue::parse),
                    |(left, right)| Declaration::Assigned { left, right },
                ),
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
            separated_pair(parse_id, wst(lexem::COLON), Type::parse),
            |(id, signature)| TypedVar { id, signature },
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
        alt((
            map(TypedVar::parse, |value| DeclaredVar::Typed(value)),
            map(PatternVar::parse, |value| DeclaredVar::Pattern(value)),
            map(parse_id, |value| DeclaredVar::Id(value)),
        ))(input)
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
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::PAR_C),
                    ),
                ),
                |((typename, variant), vars)| PatternVar::UnionInline {
                    typename,
                    variant,
                    vars,
                },
            ),
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |((typename, variant), vars)| PatternVar::UnionFields {
                    typename,
                    variant,
                    vars,
                },
            ),
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::PAR_C),
                    ),
                ),
                |(typename, vars)| PatternVar::StructInline { typename, vars },
            ),
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |(typename, vars)| PatternVar::StructFields { typename, vars },
            ),
            map(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list1(wst(lexem::COMA), parse_id),
                    wst(lexem::PAR_C),
                ),
                |value| PatternVar::Tuple(value),
            ),
        ))(input)
    }
}
#[cfg(test)]
mod tests {
    use crate::ast::{
        expressions::{
            data::{Data, Primitive, Tuple},
            Atomic, Expression,
        },
        types::PrimitiveType,
    };

    use super::*;

    #[test]
    fn valid_declaration_declared() {
        let res = Declaration::parse("let x:number;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Declared(TypedVar {
                id: "x".into(),
                signature: Type::Primitive(PrimitiveType::Number)
            }),
            value
        );
    }

    #[test]
    fn valid_declaration_assigned() {
        let res = Declaration::parse("let x:number = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Typed(TypedVar {
                    id: "x".into(),
                    signature: Type::Primitive(PrimitiveType::Number)
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );

        let res = Declaration::parse("let x = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Id("x".into()),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );

        let res = Declaration::parse("let (x,y) = (10,10);".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Declaration::Assigned {
                left: DeclaredVar::Pattern(PatternVar::Tuple(vec!["x".into(), "y".into()])),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(Data::Tuple(
                    Tuple(vec![
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(10)))),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(10))))
                    ])
                )))))
            },
            value
        );
    }
}
