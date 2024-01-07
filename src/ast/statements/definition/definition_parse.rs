use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::{
    ast::{
        statements::{declaration::TypedVar, scope::Scope},
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst},
        },
        TryParse,
    },
    semantic::scope::ScopeApi,
};

use super::{Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, TypeDef, UnionDef};

impl<Scope: ScopeApi> TryParse for Definition<Scope> {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(TypeDef::parse, |value| Definition::Type(value)),
            map(FnDef::parse, |value| Definition::Fn(value)),
            map(EventDef::parse, |value| Definition::Event(value)),
        ))(input)
    }
}

impl TryParse for TypeDef {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(StructDef::parse, |value| TypeDef::Struct(value)),
            map(UnionDef::parse, |value| TypeDef::Union(value)),
            map(EnumDef::parse, |value| TypeDef::Enum(value)),
        ))(input)
    }
}

impl TryParse for StructDef {
    /*
     * @desc Parse struct definition
     *
     * @grammar
     * Struct := struct ID { Struct_fields } | struct ID \( Types \)
     * Struct_fields := Struct_field , Struct_fields | Struct_field
     * Struct_field := ID : Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::STRUCT), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Type::parse),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(id, fields)| StructDef { id, fields },
        )(input)
    }
}

impl TryParse for UnionDef {
    /*
     * @desc Parse union definition
     *
     * @grammar
     * Union := union ID { Union_fields }
     * Union_fields := Union_field , Union_fields | Union_field
     * Union_field := ID | ID { Struct_fields } | ID \(  Types \)
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::UNION), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        pair(
                            parse_id,
                            delimited(
                                wst(lexem::BRA_O),
                                separated_list1(
                                    wst(lexem::COMA),
                                    separated_pair(parse_id, wst(lexem::COLON), Type::parse),
                                ),
                                preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                            ),
                        ),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(id, variants)| UnionDef { id, variants },
        )(input)
    }
}

impl TryParse for EnumDef {
    /*
     * @desc Parse enum definition
     *
     * @grammar
     * Enum := enum ID { Enum_fields }
     * Enum_fields := Enum_field , Enum_fields | Enum_field
     * Enum_fields := ID
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::ENUM), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(wst(lexem::COMA), parse_id),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(id, values)| EnumDef { id, values },
        )(input)
    }
}

impl<InnerScope: ScopeApi> TryParse for FnDef<InnerScope> {
    /*
     * @desc Parse function definition
     *
     * @grammar
     * FnDef := fn ID \( Fn_Params \) -> Type Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                preceded(wst(lexem::FN), parse_id),
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), TypedVar::parse),
                    wst(lexem::PAR_C),
                ),
                preceded(wst(lexem::ARROW), Type::parse),
                Scope::parse,
            )),
            |(id, params, ret, scope)| FnDef {
                id,
                params,
                ret: Box::new(ret),
                scope,
            },
        )(input)
    }
}

impl<InnerScope: ScopeApi> TryParse for EventDef<InnerScope> {
    /*
     * @desc Parse event definition
     *
     * @grammar
     * EventDef := event ID \(  EventCondition \) Scope
     * EventCondition := TODO
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                preceded(wst(lexem::EVENT), parse_id),
                EventCondition::parse,
                Scope::parse,
            )),
            |(id, condition, scope)| EventDef {
                id,
                condition,
                scope,
            },
        )(input)
    }
}

impl TryParse for EventCondition {
    fn parse(_input: Span) -> PResult<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Primitive},
                Atomic, Expression,
            },
            statements::{Return, Statement},
            types::PrimitiveType,
        },
        semantic::scope::scope_impl::MockScope,
    };

    use super::*;

    #[test]
    fn valid_struct_def() {
        let res = StructDef::parse(
            r#"
        struct Point {
            x : number,
            y : number
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            StructDef {
                id: "Point".into(),
                fields: vec![
                    ("x".into(), Type::Primitive(PrimitiveType::Number)),
                    ("y".into(), Type::Primitive(PrimitiveType::Number))
                ]
            },
            value
        );
    }

    #[test]
    fn valid_union_def() {
        let res = UnionDef::parse(
            r#"
        union Geo {
            Point {
                x : number,
                y : number
            }
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            UnionDef {
                id: "Geo".into(),
                variants: vec![(
                    "Point".into(),
                    vec![
                        ("x".into(), Type::Primitive(PrimitiveType::Number)),
                        ("y".into(), Type::Primitive(PrimitiveType::Number))
                    ]
                ),]
            },
            value
        );
    }

    #[test]
    fn valid_enum_def() {
        let res = EnumDef::parse(
            r#"
        enum Sport {
            Football,
            Basketball
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            EnumDef {
                id: "Sport".into(),
                values: vec!["Football".into(), "Basketball".into()]
            },
            value
        );
    }

    #[test]
    fn valid_fn_def() {
        let res = FnDef::<MockScope>::parse(
            r#"
        fn f(x:number) -> number {
            return 10;
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            FnDef {
                id: "f".into(),
                params: vec![TypedVar {
                    id: "x".into(),
                    signature: Type::Primitive(PrimitiveType::Number)
                }],
                ret: Box::new(Type::Primitive(PrimitiveType::Number)),
                scope: Scope {
                    instructions: vec![Statement::Return(Return::Expr(Box::new(
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(10))))
                    )))],
                    inner_scope: RefCell::new(None),
                }
            },
            value
        );
    }

    #[test]
    fn valid_event_def() {
        unimplemented!("Events condition are not implemented ");
        // let res = EventDef::parse(
        //     r#"
        // event Event( TODO ) {
        //     f(10);
        // }
        // "#
        //     .into(),
        // );
        // assert!(res.is_ok());
        // let value = res.unwrap().1;
        // assert_eq!(
        //     EventDef {
        //         id: "Event".into(),
        //         condition: todo!(),
        //         scope: Scope {
        //             instructions: vec![Statement::Flow(Flow::Call(CallStat {
        //                 fn_id: "f".into(),
        //                 params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(10)))]
        //             }))]
        //         }
        //     },
        //     value
        // );
    }
}
