use std::cell::{Cell, RefCell};

use crate::{
    ast::{
        self,
        expressions::{Atomic, Expression},
        statements::{declaration::TypedVar, return_stat::Return, Statement},
        types::NumberType,
        utils::{
            io::{PResult, Span},
            lexem,
            numbers::{parse_float, parse_number},
            strings::{
                parse_id,
                string_parser::{parse_char, parse_string},
                wst,
            },
        },
        TryParse,
    },
    semantic::{scope::ClosureState, Metadata},
    vm::{allocator::align, platform},
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, take_while_m_n},
    character::complete::digit1,
    combinator::{map, opt, peek, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    Parser,
};
use nom_supreme::error::ErrorTree;

use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, Map, MultiData, Number, Primitive,
    PtrAccess, Slice, StrSlice, Struct, Tuple, Union, Variable, Vector,
};
impl TryParse for Data {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Primitive::parse, |value| Data::Primitive(value)),
            map(StrSlice::parse, |value| Data::StrSlice(value)),
            map(Slice::parse, |value| Data::Slice(value)),
            map(Vector::parse, |value| Data::Vec(value)),
            map(Closure::parse, |value| Data::Closure(value)),
            map(Tuple::parse, |value| Data::Tuple(value)),
            map(Address::parse, |value| Data::Address(value)),
            map(PtrAccess::parse, |value| Data::PtrAccess(value)),
            value(Data::Unit, wst(lexem::UNIT)),
            map(Map::parse, |value| Data::Map(value)),
            map(Struct::parse, |value| Data::Struct(value)),
            map(Union::parse, |value| Data::Union(value)),
            map(Enum::parse, |value| Data::Enum(value)),
            map(Variable::parse, |value| Data::Variable(value)),
        ))(input)
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
        let (remainder, id) = parse_id(input)?;
        let is_in_lexem = match id.as_str() {
            lexem::ENUM => true,
            lexem::STRUCT => true,
            lexem::UNION => true,
            lexem::AS => true,
            lexem::ELSE => true,
            lexem::TRY => true,
            lexem::IF => true,
            lexem::THEN => true,
            lexem::MATCH => true,
            lexem::CASE => true,
            lexem::RETURN => true,
            lexem::LET => true,
            lexem::WHILE => true,
            lexem::FOR => true,
            lexem::LOOP => true,
            lexem::BREAK => true,
            lexem::CONTINUE => true,
            lexem::YIELD => true,
            lexem::MOVE => true,
            lexem::REC => true,
            lexem::DYN => true,
            lexem::U8 => true,
            lexem::U16 => true,
            lexem::U32 => true,
            lexem::U64 => true,
            lexem::U128 => true,
            lexem::I8 => true,
            lexem::I16 => true,
            lexem::I32 => true,
            lexem::I64 => true,
            lexem::I128 => true,
            lexem::FLOAT => true,
            lexem::CHAR => true,
            lexem::STRING => true,
            lexem::STR => true,
            lexem::BOOL => true,
            lexem::UNIT => true,
            lexem::ANY => true,
            lexem::UUNIT => true,
            lexem::UVEC => true,
            lexem::UMAP => true,
            lexem::RANGE_I => true,
            lexem::RANGE_E => true,
            lexem::GENERATOR => true,
            lexem::FN => true,
            lexem::TRUE => true,
            lexem::FALSE => true,
            _ => false,
        };
        if is_in_lexem {
            return Err(nom::Err::Error(ErrorTree::Base {
                location: remainder,
                kind: nom_supreme::error::BaseErrorKind::Kind(nom::error::ErrorKind::Fail),
            }));
        }
        Ok((
            remainder,
            Variable {
                id,
                from_field: Cell::new(false),
                metadata: Metadata::default(),
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
            map(pair(parse_float, opt(wst(lexem::FLOAT))), |(value, _)| {
                Primitive::Number(super::Number::F64(value as f64).into())
            }),
            map(Number::parse, |value| Primitive::Number(value.into())),
            map(
                alt((
                    value(true, wst(lexem::TRUE)),
                    value(false, wst(lexem::FALSE)),
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
                MultiData::parse,
                preceded(opt(wst(lexem::COMA)), wst(lexem::SQ_BRA_C)),
            ),
            |value| Slice {
                value,
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
            padding: 0.into(),
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
                wst(platform::utils::lexem::VEC),
                delimited(
                    wst(lexem::SQ_BRA_O),
                    MultiData::parse,
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
                MultiData::parse,
                preceded(opt(wst(lexem::COMA)), wst(lexem::PAR_C)),
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
                    opt(wst(lexem::MOVE)),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list0(wst(lexem::COMA), ClosureParam::parse),
                        wst(lexem::PAR_C),
                    ),
                ),
                preceded(wst(lexem::ARROW), ExprScope::parse),
            ),
            |((state, params), scope)| Closure {
                params,
                scope,
                closed: state.is_some(),
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for ExprScope {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(ast::statements::block::Block::parse, |value| {
                ExprScope::Scope(value)
            }),
            map(Expression::parse, |value| {
                ExprScope::Expr(ast::statements::block::Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(value),
                        metadata: Metadata::default(),
                    })],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                })
            }),
        ))(input)
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
        map(preceded(wst(lexem::ADDR), Atomic::parse), |value| Address {
            value: value.into(),
            metadata: Metadata::default(),
        })(input)
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
        map(preceded(wst(lexem::ACCESS), Atomic::parse), |value| {
            PtrAccess {
                value: value.into(),
                metadata: Metadata::default(),
            }
        })(input)
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
                    separated_list0(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
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
                wst(platform::utils::lexem::MAP),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        separated_pair(Expression::parse, wst(lexem::COLON), Expression::parse),
                    ),
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

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::{
        ast::{
            expressions::{
                data::Number,
                operation::{FieldAccess, FnCall, ListAccess, Range, TupleAccess},
                Atomic,
            },
            statements::block::Block,
        },
        semantic::scope::ClosureState,
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
        assert_eq!(Primitive::Number(Cell::new(Number::F64(42.5))), value);

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
                metadata: Metadata::default(),
                padding: 0.into(),
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
                    ClosureParam::Minimal("x".into()),
                    ClosureParam::Minimal("y".into())
                ],
                closed: false,
                scope: ExprScope::Expr(ast::statements::block::Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                id: "x".into(),
                                from_field: Cell::new(false),
                                metadata: Metadata::default()
                            }
                        )))),
                        metadata: Metadata::default()
                    })],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None)
                }),
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
                    ClosureParam::Minimal("x".into()),
                    ClosureParam::Minimal("y".into())
                ],
                closed: true,
                scope: ExprScope::Expr(ast::statements::block::Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                id: "x".into(),
                                from_field: Cell::new(false),
                                metadata: Metadata::default()
                            }
                        )))),
                        metadata: Metadata::default()
                    })],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None)
                }),
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
                    id: "x".into(),
                    signature: ast::types::Type::Primitive(ast::types::PrimitiveType::Number(
                        NumberType::U64
                    )),
                    rec: false,
                }),],
                closed: false,

                scope: ExprScope::Expr(ast::statements::block::Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                id: "x".into(),
                                from_field: Cell::new(false),
                                metadata: Metadata::default()
                            }
                        )))),
                        metadata: Metadata::default()
                    })],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None)
                }),
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
                    id: "x".into(),
                    from_field: Cell::new(false),
                    metadata: Metadata::default()
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
                    id: "x".into(),
                    from_field: Cell::new(false),
                    metadata: Metadata::default()
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
                            value: "x".into(),
                            padding: 0.into(),
                            metadata: Metadata::default(),
                        }))),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        Expression::Atomic(Atomic::Data(Data::StrSlice(StrSlice {
                            value: "y".into(),
                            padding: 0.into(),
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
                id: "Point".into(),
                fields: vec![
                    (
                        "x".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        "y".into(),
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
                typename: "Geo".into(),
                variant: "Point".into(),
                fields: vec![
                    (
                        "x".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        "y".into(),
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
                typename: "Geo".into(),
                value: "Point".into(),
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
                id: "x".into(),
                metadata: Metadata::default(),
                from_field: false.into()
            }))),
            value
        );

        let res = Expression::parse("x[3]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::ListAccess(ListAccess {
                var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "x".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
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
                    id: "x".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
                })))),
                field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "y".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
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
                    id: "x".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
                })))),
                field: Box::new(Expression::FieldAccess(FieldAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "y".into(),
                        metadata: Metadata::default(),
                        from_field: false.into()
                    })))),
                    field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "z".into(),
                        metadata: Metadata::default(),
                        from_field: false.into()
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
                    id: "x".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
                })))),
                field: Box::new(Expression::ListAccess(ListAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "y".into(),
                        metadata: Metadata::default(),
                        from_field: false.into()
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
                        id: "x".into(),
                        metadata: Metadata::default(),
                        from_field: false.into()
                    })))),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                        Unresolved, 3
                    ))))),
                    metadata: Metadata::default()
                })),
                field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "y".into(),
                    metadata: Metadata::default(),
                    from_field: false.into()
                })))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Expression::parse("f(10).1".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Expression::TupleAccess(TupleAccess {
                var: Expression::FnCall(FnCall {
                    lib: None,
                    fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "f".into(),
                        metadata: Metadata::default(),
                        from_field: Cell::new(false),
                    })))),
                    params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(Number::Unresolved(10).into())
                    )))],
                    metadata: Metadata::default(),
                    platform: Rc::default(),
                })
                .into(),
                index: 1,
                metadata: Metadata::default()
            }),
            value
        );
    }
}
