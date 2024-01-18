use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        self,
        expressions::Expression,
        statements::Statement,
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
    semantic::scope::ScopeApi,
};
use nom::{
    branch::alt,
    character::complete::digit1,
    combinator::{map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};
use nom_supreme::error::ErrorTree;

use super::{
    Address, Channel, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData,
    ListAccess, Map, MultiData, NumAccess, Primitive, PtrAccess, Slice, Struct, Tuple, Union,
    VarID, Variable, Vector,
};
impl<Scope: ScopeApi> TryParse for Data<Scope> {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Primitive::parse, |value| Data::Primitive(value)),
            map(Slice::parse, |value| Data::Slice(value)),
            map(Vector::parse, |value| Data::Vec(value)),
            map(Closure::parse, |value| Data::Closure(value)),
            map(Channel::parse, |value| Data::Chan(value)),
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
        map(parse_id, |id| Variable::Var(VarID(id)))(input)
    }
}

impl<InnerScope: ScopeApi> ListAccess<InnerScope> {
    fn parse(input: Span) -> PResult<Variable<InnerScope>> {
        let (remainder, var) = VarID::parse(input)?;
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
                    }),
                ));
            }
            let (remainder, right) = FieldAccess::parse(remainder)?;
            Ok((
                remainder,
                Variable::FieldAccess(FieldAccess {
                    var: Box::new(var),
                    field: Box::new(right),
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
            map(parse_float, |value| Primitive::Float(value)),
            map(parse_number, |value| Primitive::Number(value)),
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
     * @desc Parse List or Static string
     *
     * @grammar
     * Slice := \[ MultData \] | StaticString
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                delimited(
                    wst(lexem::SQ_BRA_O),
                    MultiData::parse,
                    preceded(opt(wst(lexem::COMA)), wst(lexem::SQ_BRA_C)),
                ),
                |value| Slice::List(value),
            ),
            map(parse_string, |value| Slice::String(value)),
        ))(input)
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
        alt((
            map(
                preceded(
                    wst(lexem::platform::VEC),
                    delimited(
                        wst(lexem::SQ_BRA_O),
                        MultiData::parse,
                        preceded(opt(wst(lexem::COMA)), wst(lexem::SQ_BRA_C)),
                    ),
                ),
                |value| Vector::Init(value),
            ),
            map(
                preceded(
                    wst(lexem::platform::VEC),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_pair(parse_number, wst(lexem::COMA), parse_number),
                        wst(lexem::PAR_C),
                    ),
                ),
                |(length, capacity)| Vector::Def {
                    length: length.unsigned_abs() as usize,
                    capacity: capacity.unsigned_abs() as usize,
                },
            ),
        ))(input)
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
            |value| Tuple(value),
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
                    instructions: vec![Statement::Return(ast::statements::Return::Expr(Box::new(
                        value,
                    )))],
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
            Address(value)
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
            PtrAccess(value)
        })(input)
    }
}

impl<Scope: ScopeApi> TryParse for Channel<Scope> {
    /*
     * @desc Parse grammar
     *
     * @grammar
     * ChanData := receive\[ Addr \]\(  UNumber \) | send\[ Addr \] \(  Expr \) | channel \(  StaticString \)
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                tuple((
                    wst(lexem::platform::RECEIVE),
                    delimited(wst(lexem::SQ_BRA_O), Address::parse, wst(lexem::SQ_BRA_C)),
                    delimited(wst(lexem::PAR_O), parse_number, wst(lexem::PAR_C)),
                )),
                |(_, addr, timeout)| Channel::Receive {
                    addr,
                    timeout: timeout.unsigned_abs() as usize,
                },
            ),
            map(
                tuple((
                    wst(lexem::platform::SEND),
                    delimited(wst(lexem::SQ_BRA_O), Address::parse, wst(lexem::SQ_BRA_C)),
                    delimited(wst(lexem::PAR_O), Expression::parse, wst(lexem::PAR_C)),
                )),
                |(_, addr, msg)| Channel::Send {
                    addr,
                    msg: Box::new(msg),
                },
            ),
            map(
                preceded(
                    wst(lexem::platform::CHAN),
                    delimited(wst(lexem::PAR_O), parse_string, wst(lexem::PAR_C)),
                ),
                |id| Channel::Init(id),
            ),
        ))(input)
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
            |(id, value)| Struct { id, fields: value },
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
                typename: typename,
                fields: value,
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
            |(typename, value)| Enum { typename, value },
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
        alt((
            map(
                preceded(
                    wst(lexem::platform::MAP),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(
                            wst(lexem::COMA),
                            separated_pair(KeyData::parse, wst(lexem::COLON), Expression::parse),
                        ),
                        preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                    ),
                ),
                |value| Map::Init { fields: value },
            ),
            map(
                preceded(
                    wst(lexem::platform::MAP),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_pair(parse_number, wst(lexem::COMA), parse_number),
                        wst(lexem::PAR_C),
                    ),
                ),
                |(length, capacity)| Map::Def {
                    length: length.unsigned_abs() as usize,
                    capacity: capacity.unsigned_abs() as usize,
                },
            ),
        ))(input)
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
            map(Slice::parse, |value| KeyData::Slice(value)),
            map(Primitive::parse, |value| KeyData::Primitive(value)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{expressions::Atomic, statements::Return},
        semantic::scope::scope_impl::MockScope,
    };

    use super::*;

    #[test]
    fn valid_primitive() {
        let res = Primitive::parse("true".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Bool(true), value);

        let res = Primitive::parse("false".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Bool(false), value);

        let res = Primitive::parse("42".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Number(42), value);

        let res = Primitive::parse("42.5".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Float(42.5), value);

        let res = Primitive::parse("'a'".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Char('a'), value);

        let res = Primitive::parse("'\n'".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Primitive::Char('\n'), value);
    }

    #[test]
    fn valid_slice() {
        let res = Slice::<MockScope>::parse("[2,5,6]".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Slice::List(vec![
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2)))),
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(5)))),
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(6))))
            ]),
            value
        );

        let res = Slice::<MockScope>::parse("\"Hello World\"".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Slice::String("Hello World".to_string()), value);
    }

    #[test]
    fn valid_vector() {
        let res = Vector::<MockScope>::parse("vec[2,5,6]".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Vector::Init(vec![
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2)))),
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(5)))),
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(6))))
            ]),
            value,
        );

        let res = Vector::<MockScope>::parse("vec(2,8)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Vector::Def {
                length: 2,
                capacity: 8
            },
            value
        )
    }

    #[test]
    fn valid_closure() {
        let res = Closure::<MockScope>::parse("(x,y) -> x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;

        assert_eq!(
            Closure {
                params: vec![
                    ClosureParam::Minimal("x".into()),
                    ClosureParam::Minimal("y".into())
                ],
                env: Rc::new(RefCell::new(HashMap::default())),
                scope: ExprScope::Expr(ast::statements::scope::Scope {
                    instructions: vec![Statement::Return(Return::Expr(Box::new(
                        Expression::Atomic(Atomic::Data(Data::Variable(Variable::Var(VarID(
                            "x".into(),
                        )))))
                    )))],
                    inner_scope: RefCell::new(None)
                })
            },
            value
        )
    }

    #[test]
    fn valid_chan() {
        let res = Channel::<MockScope>::parse("receive[&chan1](10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Channel::Receive {
                addr: Address(Variable::Var(VarID("chan1".into()))),
                timeout: 10
            },
            value
        );

        let res = Channel::<MockScope>::parse("send[&chan1](10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Channel::Send {
                addr: Address(Variable::Var(VarID("chan1".into()))),
                msg: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                ))))
            },
            value
        );

        let res = Channel::<MockScope>::parse("chan(\"ID_CHAN\")".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Channel::Init("ID_CHAN".into()), value);
    }

    #[test]
    fn valid_tuple() {
        let res = Tuple::<MockScope>::parse("(2,'a')".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Tuple(vec![
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2)))),
                Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Char('a'))))
            ]),
            value
        );
    }

    #[test]
    fn valid_address() {
        let res = Address::<MockScope>::parse("&x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Address(Variable::Var(VarID("x".into()))), value);
    }

    #[test]
    fn valid_access() {
        let res = PtrAccess::<MockScope>::parse("*x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(PtrAccess(Variable::Var(VarID("x".into()))), value);
    }

    #[test]
    fn valid_map() {
        let res = Map::<MockScope>::parse(r#"map{"x":2,"y":6}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Map::Init {
                fields: vec![
                    (
                        KeyData::Slice(Slice::String("x".into())),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2))))
                    ),
                    (
                        KeyData::Slice(Slice::String("y".into())),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(6))))
                    )
                ]
            },
            value
        );

        let res = Map::<MockScope>::parse(r#"map(2,8)"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Map::Def {
                length: 2,
                capacity: 8
            },
            value
        )
    }

    #[test]
    fn valid_struct() {
        let res = Struct::<MockScope>::parse(r#"Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Struct {
                id: "Point".into(),
                fields: vec![
                    (
                        "x".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2))))
                    ),
                    (
                        "y".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(8))))
                    )
                ],
            },
            value
        );
    }

    #[test]
    fn valid_union() {
        let res = Union::<MockScope>::parse(r#"Geo::Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Union {
                typename: "Geo".into(),
                variant: "Point".into(),
                fields: vec![
                    (
                        "x".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2))))
                    ),
                    (
                        "y".into(),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(8))))
                    )
                ],
            },
            value
        );
    }

    #[test]
    fn valid_enum() {
        let res = Enum::parse(r#"Geo::Point"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Enum {
                typename: "Geo".into(),
                value: "Point".into(),
            },
            value
        )
    }

    #[test]
    fn valid_variable() {
        let res = Variable::<MockScope>::parse("x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Variable::Var(VarID("x".into())), value);

        let res = Variable::<MockScope>::parse("x[3]".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Variable::ListAccess(ListAccess {
                var: Box::new(Variable::Var(VarID("x".into()))),
                index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(3)
                ))))
            }),
            value
        );

        let res = Variable::<MockScope>::parse("x.y".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID("x".into()))),
                field: Box::new(Variable::Var(VarID("y".into())))
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x.y.z".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID("x".into()))),
                field: Box::new(Variable::FieldAccess(FieldAccess {
                    var: Box::new(Variable::Var(VarID("y".into()))),
                    field: Box::new(Variable::Var(VarID("z".into())))
                }))
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x.y[3]".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::Var(VarID("x".into()))),
                field: Box::new(Variable::ListAccess(ListAccess {
                    var: Box::new(Variable::Var(VarID("y".into()))),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(3)
                    ))))
                }))
            }),
            value
        );
        let res = Variable::<MockScope>::parse("x[3].y".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Variable::FieldAccess(FieldAccess {
                var: Box::new(Variable::ListAccess(ListAccess {
                    var: Box::new(Variable::Var(VarID("x".into()))),
                    index: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                        Primitive::Number(3)
                    ))))
                })),
                field: Box::new(Variable::Var(VarID("y".into())))
            }),
            value
        );
    }
}
