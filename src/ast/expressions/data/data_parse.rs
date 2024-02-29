use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        self,
        expressions::Expression,
        statements::{return_stat::Return, Statement},
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
    semantic::{scope::ScopeApi, Metadata},
    vm::{allocator::align, platform},
};
use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::digit1,
    combinator::{map, opt, peek, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};
use nom_supreme::error::ErrorTree;

use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData, ListAccess, Map,
    MultiData, NumAccess, Primitive, PtrAccess, Slice, StrSlice, Struct, Tuple, Union, VarID,
    Variable, Vector,
};
impl<Scope: ScopeApi> TryParse for Data<Scope> {
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

impl VarID {
    fn parse<InnerScope: ScopeApi>(input: Span) -> PResult<Variable<InnerScope>> {
        map(parse_id, |id| {
            Variable::Var(VarID {
                id,
                metadata: Metadata::default(),
            })
        })(input)
    }
}

impl<InnerScope: ScopeApi> NumAccess<InnerScope> {
    fn parse(input: Span) -> PResult<Variable<InnerScope>> {
        let (remainder, var) = VarID::parse(input)?;
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
                        Variable::NumAccess(NumAccess {
                            var: Box::new(var),
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
            Ok((remainder, var))
        }
    }
}

impl<InnerScope: ScopeApi> ListAccess<InnerScope> {
    fn parse(input: Span) -> PResult<Variable<InnerScope>> {
        let (remainder, var) = NumAccess::parse(input)?;
        let (remainder, index) = opt(delimited(
            wst(lexem::SQ_BRA_O),
            Expression::parse,
            wst(lexem::SQ_BRA_C),
        ))(remainder)?;

        if let Some(index) = index {
            Ok((
                remainder,
                Variable::ListAccess(ListAccess {
                    var: Box::new(var),
                    index: Box::new(index),
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, var))
        }
    }
}

impl<InnerScope: ScopeApi> FieldAccess<InnerScope> {
    fn parse(input: Span) -> PResult<Variable<InnerScope>> {
        let (remainder, var) = ListAccess::parse(input)?;
        let (remainder, index) = opt(wst(lexem::DOT))(remainder)?;

        if let Some(_index) = index {
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
                    Variable::NumAccess(NumAccess {
                        var: Box::new(var),
                        index,
                        metadata: Metadata::default(),
                    }),
                ));
            }
            let (remainder, right) = FieldAccess::parse(remainder)?;
            Ok((
                remainder,
                Variable::FieldAccess(FieldAccess {
                    var: Box::new(var),
                    field: Box::new(right),
                    metadata: Metadata::default(),
                }),
            ))
        } else {
            Ok((remainder, var))
        }
    }
}

impl<InnerScope: ScopeApi> TryParse for Variable<InnerScope> {
    /*
     * @desc Parse variable
     *
     * @grammar
     * Variable := ID | Variable . Variable | Variable [ num ]
     */
    fn parse(input: Span) -> PResult<Self> {
        FieldAccess::parse(input)
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
            map(
                pair(parse_number, opt(NumberType::parse)),
                |(value, opt_type)| match opt_type {
                    Some(number_type) => match number_type {
                        NumberType::U8 => Primitive::Number(super::Number::U8(value as u8).into()),
                        NumberType::U16 => {
                            Primitive::Number(super::Number::U16(value as u16).into())
                        }
                        NumberType::U32 => {
                            Primitive::Number(super::Number::U32(value as u32).into())
                        }
                        NumberType::U64 => {
                            Primitive::Number(super::Number::U64(value as u64).into())
                        }
                        NumberType::U128 => {
                            Primitive::Number(super::Number::U128(value as u128).into())
                        }
                        NumberType::I8 => Primitive::Number(super::Number::I8(value as i8).into()),
                        NumberType::I16 => {
                            Primitive::Number(super::Number::I16(value as i16).into())
                        }
                        NumberType::I32 => {
                            Primitive::Number(super::Number::I32(value as i32).into())
                        }
                        NumberType::I64 => {
                            Primitive::Number(super::Number::I64(value as i64).into())
                        }
                        NumberType::I128 => {
                            Primitive::Number(super::Number::I128(value as i128).into())
                        }
                        NumberType::F64 => {
                            Primitive::Number(super::Number::F64(value as f64).into())
                        }
                    },
                    None => Primitive::Number(super::Number::Unresolved(value as i64).into()),
                },
            ),
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

impl<Scope: ScopeApi> TryParse for Slice<Scope> {
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
            metadata: Metadata::default(),
        })(input)
    }
}

impl<Scope: ScopeApi> TryParse for Vector<Scope> {
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

impl<Scope: ScopeApi> TryParse for Tuple<Scope> {
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

impl<Scope: ScopeApi> TryParse for MultiData<Scope> {
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

impl<Scope: ScopeApi> TryParse for Closure<Scope> {
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), ClosureParam::parse),
                    wst(lexem::PAR_C),
                ),
                preceded(wst(lexem::ARROW), ExprScope::parse),
            ),
            |(params, scope)| Closure {
                params,
                scope,
                env: Rc::new(RefCell::new(HashMap::default())),
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl<Scope: ScopeApi> TryParse for ExprScope<Scope> {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(ast::statements::scope::Scope::parse, |value| {
                ExprScope::Scope(value)
            }),
            map(Expression::parse, |value| {
                ExprScope::Expr(ast::statements::scope::Scope {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(value),
                        metadata: Metadata::default(),
                    })],
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
            map(parse_id, |value| ClosureParam::Minimal(value)),
            map(
                separated_pair(parse_id, wst(lexem::COLON), ast::types::Type::parse),
                |(id, signature)| ClosureParam::Full { id, signature },
            ),
        ))(input)
    }
}

impl<InnerScope: ScopeApi> TryParse for Address<InnerScope> {
    /*
     * @desc Parse pointer address
     *
     * @grammar
     * Addr := &DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(preceded(wst(lexem::ADDR), Variable::parse), |value| {
            Address {
                value,
                metadata: Metadata::default(),
            }
        })(input)
    }
}

impl<InnerScope: ScopeApi> TryParse for PtrAccess<InnerScope> {
    /*
     * @desc Parse pointer access
     *
     * @grammar
     * Addr := -DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(preceded(wst(lexem::ACCESS), Variable::parse), |value| {
            PtrAccess {
                value,
                metadata: Metadata::default(),
            }
        })(input)
    }
}

impl<Scope: ScopeApi> TryParse for Struct<Scope> {
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
                    separated_list1(
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

impl<Scope: ScopeApi> TryParse for Union<Scope> {
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
                    separated_list1(
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

impl<Scope: ScopeApi> TryParse for Map<Scope> {
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
                        separated_pair(KeyData::parse, wst(lexem::COLON), Expression::parse),
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

impl<Scope: ScopeApi> TryParse for KeyData<Scope> {
    /*
     * @desc Parse hashable data for key
     *
     * @grammar
     * KeyData := Number | Bool | Char | String | Addr | EnumData | StaticString
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Address::parse, |value| KeyData::Address(value)),
            map(Enum::parse, |value| KeyData::Enum(value)),
            map(Primitive::parse, |value| KeyData::Primitive(value)),
            map(StrSlice::parse, |value| KeyData::StrSlice(value)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::expressions::{data::Number, Atomic},
        semantic::scope::scope_impl::MockScope,
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
        assert_eq!(Primitive::Number(Cell::new(Number::Unresolved(42))), value);

        let res = Primitive::parse("42i16".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Primitive::Number(Cell::new(Number::I16(42))), value);

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
        let res = Slice::<MockScope>::parse("[2,5,6]".into());
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
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_vector() {
        let res = Vector::<MockScope>::parse("vec[2,5,6]".into());
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
        let res = Closure::<MockScope>::parse("(x,y) -> x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;

        assert_eq!(
            Closure {
                params: vec![
                    ClosureParam::Minimal("x".into()),
                    ClosureParam::Minimal("y".into())
                ],
                env: Rc::new(RefCell::new(HashMap::default())),
                scope: ExprScope::Expr(ast::statements::scope::Scope {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable::Var(VarID {
                                id: "x".into(),
                                metadata: Metadata::default()
                            })
                        )))),
                        metadata: Metadata::default()
                    })],
                    inner_scope: RefCell::new(None)
                }),
                metadata: Metadata::default()
            },
            value
        )
    }

    #[test]
    fn valid_tuple() {
        let res = Tuple::<MockScope>::parse("(2,'a')".into());
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
        let res = Address::<MockScope>::parse("&x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Address {
                value: Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                }),
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_access() {
        let res = PtrAccess::<MockScope>::parse("*x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            PtrAccess {
                value: Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                }),
                metadata: Metadata::default()
            },
            value
        );
    }

    #[test]
    fn valid_map() {
        let res = Map::<MockScope>::parse(r#"map{"x":2,"y":6}"#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Map {
                fields: vec![
                    (
                        KeyData::StrSlice(StrSlice {
                            value: "x".into(),
                            metadata: Metadata::default(),
                        }),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                            Number::Unresolved(2).into()
                        ))))
                    ),
                    (
                        KeyData::StrSlice(StrSlice {
                            value: "y".into(),
                            metadata: Metadata::default(),
                        }),
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
        let res = Struct::<MockScope>::parse(r#"Point { x : 2, y : 8}"#.into());
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
        let res = Union::<MockScope>::parse(r#"Geo::Point { x : 2, y : 8}"#.into());
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
        let res = Variable::<MockScope>::parse("x".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::Var(VarID {
                id: "x".into(),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Variable::<MockScope>::parse("x[3]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::ListAccess(ListAccess {
                var: Box::new(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(Cell::new(Number::Unresolved(3)))
                )))),
                metadata: Metadata::default()
            }),
            value
        );

        let res = Variable::<MockScope>::parse("x.y".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                field: Box::new(Variable::Var(VarID {
                    id: "y".into(),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x.y.z".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                field: Box::new(Variable::FieldAccess(FieldAccess {
                    var: Box::new(Variable::Var(VarID {
                        id: "y".into(),
                        metadata: Metadata::default()
                    })),
                    field: Box::new(Variable::Var(VarID {
                        id: "z".into(),
                        metadata: Metadata::default()
                    })),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x.y[3]".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                field: Box::new(Variable::ListAccess(ListAccess {
                    var: Box::new(Variable::Var(VarID {
                        id: "y".into(),
                        metadata: Metadata::default()
                    })),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(Cell::new(Number::Unresolved(3)))
                    )))),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x[3].y".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::ListAccess(ListAccess {
                    var: Box::new(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(Cell::new(Number::Unresolved(3)))
                    )))),
                    metadata: Metadata::default()
                })),
                field: Box::new(Variable::Var(VarID {
                    id: "y".into(),
                    metadata: Metadata::default()
                })),
                metadata: Metadata::default()
            }),
            value
        );
    }
}
