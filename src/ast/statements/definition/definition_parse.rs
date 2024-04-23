

use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};


use crate::ast::{
    statements::{block::Block, declaration::TypedVar},
    types::Type,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst},
    },
    TryParse,
};

use super::{Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, TypeDef, UnionDef};

impl TryParse for Definition {
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
                    separated_list0(
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
                                separated_list0(
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

impl TryParse for FnDef {
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
                Block::parse,
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

impl TryParse for EventDef {
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
                Block::parse,
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

    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
    };

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive},
                Atomic, Expression,
            },
            statements::{Return, Statement},
            types::{NumberType, PrimitiveType},
        },
        semantic::{scope::ClosureState, Metadata},
        v_num,
    };

    use super::*;

    #[test]
    fn valid_struct_def() {
        let res = StructDef::parse(
            r#"
        struct Point {
            x : u64,
            y : u64
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            StructDef {
                id: "Point".into(),
                fields: vec![
                    (
                        "x".into(),
                        Type::Primitive(PrimitiveType::Number(NumberType::U64))
                    ),
                    (
                        "y".into(),
                        Type::Primitive(PrimitiveType::Number(NumberType::U64))
                    )
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
                x : u64,
                y : u64
            }
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            UnionDef {
                id: "Geo".into(),
                variants: vec![(
                    "Point".into(),
                    vec![
                        (
                            "x".into(),
                            Type::Primitive(PrimitiveType::Number(NumberType::U64))
                        ),
                        (
                            "y".into(),
                            Type::Primitive(PrimitiveType::Number(NumberType::U64))
                        )
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
        assert!(res.is_ok(), "{:?}", res);
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
        let res = FnDef::parse(
            r#"
        fn f(x:u64) -> u64 {
            return 10;
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            FnDef {
                id: "f".into(),
                params: vec![TypedVar {
                    id: "x".into(),
                    rec: false,
                    signature: Type::Primitive(PrimitiveType::Number(NumberType::U64))
                }],
                ret: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::U64))),
                scope: Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))),
                        metadata: Metadata::default()
                    })],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
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
        // assert!(res.is_ok(), "{:?}", res);
        // let value = res.unwrap().1;
        // assert_eq!(
        //     EventDef {
        //         id: "Event".into(),
        //         condition: todo!(),
        //         block: Scope {
        //             instructions: vec![Statement::Flow(Flow::Call(CallStat {
        //                 fn_id: "f".into(),
        //                 params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(I64, 10)))]
        //             }))]
        //         }
        //     },
        //     value
        // );
    }
}
