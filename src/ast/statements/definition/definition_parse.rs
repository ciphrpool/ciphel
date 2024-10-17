use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::ast::{
    statements::{block::FunctionBlock, declaration::TypedVar},
    types::Type,
    utils::{
        error::squash,
        io::{PResult, Span},
        lexem,
        numbers::{parse_number, parse_number_u64},
        strings::{parse_id, wst, wst_closed},
    },
    TryParse,
};

use super::{Definition, EnumDef, FnDef, StructDef, TypeDef, UnionDef};

impl TryParse for Definition {
    fn parse(input: Span) -> PResult<Self> {
        squash(
            alt((
                map(TypeDef::parse, |value| Definition::Type(value)),
                map(FnDef::parse, |value| Definition::Fn(value)),
            )),
            "Expected a type definition (a struct, enum or union) or a static function definition",
        )(input)
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
                preceded(wst_closed(lexem::STRUCT), parse_id),
                cut(delimited(
                    wst(lexem::BRA_O),
                    separated_list0(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Type::parse),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                )),
            ),
            |(id, fields)| StructDef {
                id,
                fields,
                signature: None,
            },
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
                preceded(wst_closed(lexem::UNION), parse_id),
                cut(delimited(
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
                )),
            ),
            |(id, variants)| UnionDef {
                id,
                variants,
                signature: None,
            },
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
                preceded(wst_closed(lexem::ENUM), parse_id),
                cut(delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        pair(parse_id, opt(preceded(wst(lexem::EQUAL), parse_number_u64))),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                )),
            ),
            |(id, opt_values)| {
                let mut values = Vec::with_capacity(opt_values.len());
                let mut current: Option<u64> = None;
                for (name, value) in opt_values {
                    if let Some(v) = value {
                        values.push((name, v));
                        let _ = current.insert(v + 1);
                    } else {
                        let c = current.unwrap_or(0);
                        values.push((name, c));
                        let _ = current.insert(c + 1);
                    }
                }
                EnumDef {
                    id,
                    values,
                    signature: None,
                }
            },
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
                preceded(wst_closed(lexem::FN), parse_id),
                cut(delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), TypedVar::parse),
                    wst(lexem::PAR_C),
                )),
                map(
                    opt(preceded(wst(lexem::ARROW), Type::parse)),
                    |opt_return_type| opt_return_type.unwrap_or(Type::Unit),
                ),
                FunctionBlock::parse,
            )),
            |(name, params, ret, scope)| FnDef {
                name,
                id: None,
                params,
                ret: Box::new(ret),
                scope,
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{data::Data, Atomic, Expression},
            statements::{block::FunctionBlock, Return, Statement},
            types::{NumberType, PrimitiveType},
        },
        semantic::Metadata,
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
                id: "Point".to_string().into(),
                fields: vec![
                    (
                        "x".to_string().into(),
                        Type::Primitive(PrimitiveType::Number(NumberType::U64))
                    ),
                    (
                        "y".to_string().into(),
                        Type::Primitive(PrimitiveType::Number(NumberType::U64))
                    )
                ],
                signature: None,
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
                id: "Geo".to_string().into(),
                variants: vec![(
                    "Point".to_string().into(),
                    vec![
                        (
                            "x".to_string().into(),
                            Type::Primitive(PrimitiveType::Number(NumberType::U64))
                        ),
                        (
                            "y".to_string().into(),
                            Type::Primitive(PrimitiveType::Number(NumberType::U64))
                        )
                    ]
                ),],
                signature: None,
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
                id: "Sport".to_string(),
                values: vec![("Football".to_string(), 0), ("Basketball".to_string(), 1)],
                signature: None,
            },
            value
        );

        let res = EnumDef::parse(
            r#"
        enum Sport {
            Football = 2,
            Basketball = 3,
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            EnumDef {
                id: "Sport".to_string(),
                values: vec![("Football".to_string(), 2), ("Basketball".to_string(), 3)],
                signature: None,
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
                name: "f".to_string(),
                id: None,
                params: vec![TypedVar {
                    name: "x".to_string(),
                    id: None,
                    signature: Type::Primitive(PrimitiveType::Number(NumberType::U64))
                }],
                ret: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::U64))),
                scope: FunctionBlock::new(vec![Statement::Return(Return::Expr {
                    expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 10
                    ))))),
                    metadata: Metadata::default()
                })])
            },
            value
        );
    }
}
