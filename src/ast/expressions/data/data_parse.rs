use std::sync::{Arc, RwLock};

use crate::{
    ast::{
        self,
        expressions::{Atomic, CompletePath, Expression},
        statements::{
            block::{ClosureBlock, LambdaBlock},
            declaration::TypedVar,
            return_stat::Return,
            Statement,
        },
        types::NumberType,
        utils::{
            error::squash,
            io::{PResult, Span},
            lexem,
            numbers::{parse_float, parse_number},
            strings::{
                parse_id,
                string_parser::{parse_char, parse_string},
                wst, wst_closed,
            },
        },
        TryParse,
    },
    semantic::Metadata,
    vm::{
        allocator::{align, MemoryAddress},
        core,
    },
};
use nom::{
    branch::alt,
    combinator::{cut, map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
    Parser,
};
use nom_supreme::{final_parser::ExtractContext, ParserExt};

use super::{
    Address, Call, CallArgs, Closure, ClosureParam, Data, Enum, Lambda, LeftCall, Map, MultiData,
    Number, Primitive, PtrAccess, Slice, StrSlice, Struct, Tuple, Union, VarCall, Variable, Vector,
};
impl TryParse for Data {
    fn parse(input: Span) -> PResult<Self> {
        squash(
            alt((
                map(Primitive::parse, |value| Data::Primitive(value)),
                map(StrSlice::parse, |value| Data::StrSlice(value)),
                map(Slice::parse, |value| Data::Slice(value)),
                map(Vector::parse, |value| Data::Vec(value)),
                map(Closure::parse, |value| Data::Closure(value)),
                map(Lambda::parse, |value| Data::Lambda(value)),
                map(Tuple::parse, |value| Data::Tuple(value)),
                map(Address::parse, |value| Data::Address(value)),
                map(PtrAccess::parse, |value| Data::PtrAccess(value)),
                value(Data::Unit, wst_closed(lexem::UNIT)),
                map(Map::parse, |value| Data::Map(value)),
                map(Call::parse, |value| Data::Call(value)),
                map(Struct::parse, |value| Data::Struct(value)),
                map(Union::parse, |value| Data::Union(value)),
                map(Enum::parse, |value| Data::Enum(value)),
                map(Variable::parse, |value| Data::Variable(value)),
            )),
            "Expected a valid expression",
        )(input)
    }
}

impl TryParse for Variable {
    /*
     * @desc Parse variable
     *
     * @grammar
     * Variable := ID
     */
    fn parse(input: Span) -> PResult<Self> {
        let (remainder, name) = parse_id(input)?;
        Ok((
            remainder,
            Variable {
                name,
                metadata: Metadata::default(),
                state: None,
            },
        ))
    }
}

impl TryParse for Number {
    fn parse(input: Span) -> PResult<Self>
    where
        Self: Sized,
    {
        map(
            pair(parse_number, opt(NumberType::parse)),
            |(value, opt_type)| match opt_type {
                Some(number_type) => match number_type {
                    NumberType::U8 => super::Number::U8(value as u8).into(),
                    NumberType::U16 => super::Number::U16(value as u16).into(),
                    NumberType::U32 => super::Number::U32(value as u32).into(),
                    NumberType::U64 => super::Number::U64(value as u64).into(),
                    NumberType::U128 => super::Number::U128(value as u128).into(),
                    NumberType::I8 => super::Number::I8(value as i8).into(),
                    NumberType::I16 => super::Number::I16(value as i16).into(),
                    NumberType::I32 => super::Number::I32(value as i32).into(),
                    NumberType::I64 => super::Number::I64(value as i64).into(),
                    NumberType::I128 => super::Number::I128(value as i128).into(),
                    NumberType::F64 => super::Number::F64(value as f64).into(),
                },
                None => super::Number::Unresolved(value as i64).into(),
            },
        )(input)
    }
}

impl TryParse for Primitive {
    /*
     * @desc Parse Primitive data
     *
     * @grammar
     * Primitive := Number | true | false | Char | Float
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                pair(parse_float, opt(wst_closed(lexem::FLOAT))),
                |(value, _)| Primitive::Number(super::Number::F64(value as f64).into()),
            ),
            map(Number::parse, |value| Primitive::Number(value.into())),
            map(
                alt((
                    value(true, wst_closed(lexem::TRUE)),
                    value(false, wst_closed(lexem::FALSE)),
                )),
                |value| Primitive::Bool(value),
            ),
            map(parse_char, |value| Primitive::Char(value)),
        ))(input)
    }
}

impl TryParse for Slice {
    /*
     * @desc Parse List
     *
     * @grammar
     * Slice := \[ MultData \]
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::SQ_BRA_O),
                cut(MultiData::parse).context("Invalid slice"),
                cut(preceded(opt(wst(lexem::COMA)), wst(lexem::SQ_BRA_C))),
            ),
            |value| Slice {
                value,
                size: 0,
                address: None,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for StrSlice {
    /*
     * @desc Parse Static string
     *
     * @grammar
     * StrSlice := StaticString
     */
    fn parse(input: Span) -> PResult<Self> {
        map(parse_string, |value| StrSlice {
            value,
            address: None,
            metadata: Metadata::default(),
        })(input)
    }
}

impl TryParse for Vector {
    /*
     * @desc Parse Vector
     *
     * @grammar
     * Vector := vec\[ MultData1\] | vec( UNumber, UNumber )
     *
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(core::lexem::VEC),
                delimited(
                    wst(lexem::SQ_BRA_O),
                    cut(MultiData::parse).context("Invalid vector"),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::SQ_BRA_C)),
                ),
            ),
            |value| Vector {
                capacity: align(*&value.len()),
                length: *&value.len(),
                value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Tuple {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::PAR_O),
                cut(MultiData::parse).context("Invalid tuple"),
                cut(preceded(opt(wst(lexem::COMA)), wst(lexem::PAR_C))),
            ),
            |value| Tuple {
                value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for MultiData {
    /*
     * @desc Parse multiple data
     *
     * @grammar
     * MultData1 := Data , MultData1 | Data
     *
     */
    fn parse(input: Span) -> PResult<Self> {
        separated_list0(wst(lexem::COMA), Expression::parse)(input)
    }
}

impl TryParse for Closure {
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                pair(
                    wst_closed(lexem::MOVE),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list0(wst(lexem::COMA), ClosureParam::parse),
                        wst(lexem::PAR_C),
                    ),
                ),
                preceded(wst(lexem::ARROW), cut(ClosureBlock::parse)).context("Invalid closure"),
            ),
            |((_, params), block)| Closure {
                params,
                block,
                repr_data: None,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Lambda {
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), ClosureParam::parse),
                    wst(lexem::PAR_C),
                ),
                cut(preceded(wst(lexem::ARROW), cut(LambdaBlock::parse))).context("Invalid lambda"),
            ),
            |(params, block)| Lambda {
                params,
                block,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for ClosureParam {
    /*
     * @desc Parse Closure parameters
     *
     * @grammar
     *  Params := ID | ID : Type
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(TypedVar::parse, |var| ClosureParam::Full(var)),
            map(parse_id, |value| ClosureParam::Minimal(value)),
        ))(input)
    }
}

impl TryParse for Address {
    /*
     * @desc Parse pointer address
     *
     * @grammar
     * Addr := &DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::ADDR),
                cut(Atomic::parse).context("Invalid address"),
            ),
            |value| Address {
                value: value.into(),
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for PtrAccess {
    /*
     * @desc Parse pointer access
     *
     * @grammar
     * Addr := -DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::ACCESS),
                cut(Atomic::parse).context("Invalid pointer access"),
            ),
            |value| PtrAccess {
                value: value.into(),
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Struct {
    /*
     * @desc Parse an object
     *
     * @grammar
     * Object := ID { ObjFields } | ID ( MultiData1 )
     * ObjFields := ObjField , ObjFields | ObjField
     * ObjField := ID | ID : Expr
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                parse_id,
                delimited(
                    wst(lexem::BRA_O),
                    separated_list0(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(id, value)| Struct {
                id,
                fields: value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Union {
    /*
     * @desc Parse an union
     *
     * @grammar
     * Object := ID { ObjFields } | ID ( MultiData1 )
     * ObjFields := ObjField , ObjFields | ObjField
     * ObjField := ID | ID : Expr
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                separated_pair(parse_id, wst(lexem::SEP), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    cut(separated_list0(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                    )),
                    cut(preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C))),
                ),
            ),
            |((typename, name), value)| Union {
                variant: name,
                typename,
                fields: value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Enum {
    /*
     * @desc Parse enum
     *
     * @grammar
     * EnumData := ID :: ID
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(parse_id, wst(lexem::SEP), parse_id),
            |(typename, value)| Enum {
                typename,
                value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Map {
    /*
     * @desc Parse map
     *
     * @grammar
     * MapData := map { MapFields }  | map( UNumber, UNumber )
     * MapFields := MapField , MapFields | Λ
     * MapField := KeyData : Data
     * KeyData := Number | Bool | Char | String | Addr | EnumData | StaticString
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(core::lexem::MAP),
                delimited(
                    wst(lexem::BRA_O),
                    cut(separated_list1(
                        wst(lexem::COMA),
                        separated_pair(Expression::parse, wst(lexem::COLON), Expression::parse),
                    )),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |value| Map {
                fields: value,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for CallArgs {
    /*
     * @desc Parse variable
     *
     * @grammar
     * CallArgs := (( Expression, )*)
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::PAR_O),
                separated_list0(wst(lexem::COMA), Expression::parse),
                wst(lexem::PAR_C),
            ),
            |args| CallArgs { args, size: None },
        )(input)
    }
}

impl TryParse for Call {
    /*
     * @desc Parse variable
     *
     * @grammar
     * Call := CompletePath (CallArgs)
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(CompletePath::parse, CallArgs::parse),
            |(path, args)| Call {
                path: LeftCall::VarCall(VarCall {
                    path,
                    id: None,
                    is_closure: false,
                }),
                args,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        ast::expressions::{
            data::Number,
            operation::{FieldAccess, ListAccess, TupleAccess},
            Atomic,
        },
        v_num,
    };

    use super::*;

    #[test]
    fn valid_primitive() {
        let res = Primitive::parse("true".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Bool(true), value);

        let res = Primitive::parse("false".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Bool(false), value);

        let res = Primitive::parse("42".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(v_num!(Unresolved, 42), value);

        let res = Primitive::parse("42i16".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(v_num!(I16, 42), value);

        let res = Primitive::parse("42.5".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Number(Number::F64(42.5)), value);

        let res = Primitive::parse("'a'".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Char('a'), value);

        let res = Primitive::parse("'\n'".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Char('\n'), value);
    }

    #[test]
    fn valid_slice() {
        let res = Slice::parse("[2,5,6]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Slice {
                value: vec![
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(2).into()
                    )))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(5).into()
                    )))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(6).into()
                    ))))
                ],
                size: 0,
                address: None,
                metadata: Metadata::default(),
            },
            value
        );
    }

    #[test]
    fn valid_str_slice() {
        let res = StrSlice::parse("\"Hello World\"".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            StrSlice {
                value: "Hello World".to_string(),
                address: None,
                metadata: Metadata::default(),
            },
            value
        );
    }

    #[test]
    fn valid_vector() {
        let res = Vector::parse("vec[2,5,6]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Vector {
                value: vec![
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(2).into()
                    )))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(5).into()
                    )))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(6).into()
                    ))))
                ],
                capacity: align(3),
                length: 3,
                metadata: Metadata::default(),
            },
            value,
        );
    }

    #[test]
    fn valid_closure() {
        let res = Closure::parse("(x,y) -> x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;

        assert_eq!(
            Closure {
                params: vec![
                    ClosureParam::Minimal("x".to_string().into()),
                    ClosureParam::Minimal("y".to_string().into())
                ],
                block: ast::statements::block::ClosureBlock::new(vec![Statement::Return(
                    Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                name: "x".to_string().into(),
                                metadata: Metadata::default(),
                                state: None,
                            }
                        )))),
                        metadata: Metadata::default()
                    }
                )]),
                repr_data: None,
                metadata: Metadata::default()
            },
            value
        )
    }

    #[test]
    fn valid_closure_closed() {
        let res = Closure::parse("move (x,y) -> x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;

        assert_eq!(
            Closure {
                params: vec![
                    ClosureParam::Minimal("x".to_string().into()),
                    ClosureParam::Minimal("y".to_string().into())
                ],
                block: ast::statements::block::ClosureBlock::new(vec![Statement::Return(
                    Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                name: "x".to_string().into(),
                                metadata: Metadata::default(),
                                state: None,
                            }
                        )))),
                        metadata: Metadata::default()
                    }
                )]),
                repr_data: None,
                metadata: Metadata::default()
            },
            value
        )
    }
    #[test]
    fn valid_closure_typed_args() {
        let res = Closure::parse("(x:u64) -> x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;

        assert_eq!(
            Closure {
                params: vec![ClosureParam::Full(TypedVar {
                    name: "x".to_string(),
                    id: None,
                    signature: ast::types::Type::Primitive(ast::types::PrimitiveType::Number(
                        NumberType::U64
                    )),
                }),],
                block: ast::statements::block::ClosureBlock::new(vec![Statement::Return(
                    Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                name: "x".to_string().into(),
                                metadata: Metadata::default(),
                                state: None,
                            }
                        )))),
                        metadata: Metadata::default()
                    }
                )]),
                repr_data: None,
                metadata: Metadata::default()
            },
            value
        )
    }
    #[test]
    fn valid_tuple() {
        let res = Tuple::parse("(2,'a')".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Tuple {
                value: vec![
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(2).into()
                    )))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Char('a'))))
                ],
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_address() {
        let res = Address::parse("&x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Address {
                value: Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                }))
                .into(),
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_access() {
        let res = PtrAccess::parse("*x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            PtrAccess {
                value: Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                }))
                .into(),
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_map() {
        let res = Map::parse(r#"map{"x":2,"y":6}"#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Map {
                fields: vec![
                    (
                        Expression::Atomic(Atomic::Data(Data::StrSlice(StrSlice {
                            value: "x".to_string().into(),
                            address: None,
                            metadata: Metadata::default(),
                        }))),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        Expression::Atomic(Atomic::Data(Data::StrSlice(StrSlice {
                            value: "y".to_string().into(),
                            address: None,
                            metadata: Metadata::default(),
                        }))),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(6).into()
                        ))))
                    )
                ],
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_struct() {
        let res = Struct::parse(r#"Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Struct {
                id: "Point".to_string().into(),
                fields: vec![
                    (
                        "x".to_string().into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        "y".to_string().into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(8).into()
                        ))))
                    )
                ],
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_union() {
        let res = Union::parse(r#"Geo::Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Union {
                typename: "Geo".to_string().into(),
                variant: "Point".to_string().into(),
                fields: vec![
                    (
                        "x".to_string().into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        "y".to_string().into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(8).into()
                        ))))
                    )
                ],
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_enum() {
        let res = Enum::parse(r#"Geo::Point"#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Enum {
                typename: "Geo".to_string().into(),
                value: "Point".to_string().into(),
                metadata: Metadata::default()
            },
            value
        )
    }

    #[test]
    fn valid_variable() {
        let res = Expression::parse("x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                name: "x".to_string().into(),
                metadata: Metadata::default(),
                state: None,
            }))),
            value
        );

        let res = Expression::parse("x[3]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::ListAccess(ListAccess {
                var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                    Unresolved, 3
                ))))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Expression::parse("x.y".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FieldAccess(FieldAccess {
                var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "y".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Expression::parse("x.y.z".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FieldAccess(FieldAccess {
                var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                field: Box::new(Expression::FieldAccess(FieldAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        name: "y".to_string().into(),
                        metadata: Metadata::default(),
                        state: None,
                    })))),
                    field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        name: "z".to_string().into(),
                        metadata: Metadata::default(),
                        state: None,
                    })))),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Expression::parse("x.y[3]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FieldAccess(FieldAccess {
                var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "x".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                field: Box::new(Expression::ListAccess(ListAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        name: "y".to_string().into(),
                        metadata: Metadata::default(),
                        state: None,
                    })))),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 3
                    ))))),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Expression::parse("x[3].y".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::FieldAccess(FieldAccess {
                var: Box::new(Expression::ListAccess(ListAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        name: "x".to_string().into(),
                        metadata: Metadata::default(),
                        state: None,
                    })))),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 3
                    ))))),
                    metadata: Metadata::default()
                })),
                field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    name: "y".to_string().into(),
                    metadata: Metadata::default(),
                    state: None,
                })))),
                metadata: Metadata::default()
            }),
            value
        );
    }
}
