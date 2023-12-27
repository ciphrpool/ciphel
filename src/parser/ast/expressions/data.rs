use std::collections::HashMap;

use nom::{
    branch::alt,
    combinator::{cut, map, value},
    multi::{fold_many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::parser::{
    ast::{self, types::Type, TryParse},
    utils::{
        io::{PResult, Span},
        lexem,
        numbers::{parse_float, parse_number, Number},
        strings::{
            eater, parse_id,
            string_parser::{parse_char, parse_string},
            wst, ID,
        },
    },
};

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Primitive(Primitive),
    Slice(Slice),
    Vec(Vector),
    Closure(Closure),
    Chan(Channel),
    Tuple(Tuple),
    Address(Address),
    Access(Access),
    Variable(Variable),
    Unit,
    Map(Map),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
}

impl TryParse for Data {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Primitive::parse, |value| Data::Primitive(value)),
            map(Slice::parse, |value| Data::Slice(value)),
            map(Vector::parse, |value| Data::Vec(value)),
            map(Closure::parse, |value| Data::Closure(value)),
            map(Channel::parse, |value| Data::Chan(value)),
            map(Tuple::parse, |value| Data::Tuple(value)),
            map(Address::parse, |value| Data::Address(value)),
            map(Access::parse, |value| Data::Access(value)),
            value(Data::Unit, wst(lexem::UNIT)),
            map(Map::parse, |value| Data::Map(value)),
            map(Struct::parse, |value| Data::Struct(value)),
            map(Union::parse, |value| Data::Union(value)),
            map(Enum::parse, |value| Data::Enum(value)),
            map(Variable::parse, |value| Data::Variable(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Var(ID),
    FieldAccess(Vec<ID>),
}

impl TryParse for Variable {
    /*
     * @desc Parse variable
     *
     * @grammar
     * Variable := ID | (ID . ) * ID
     */
    fn parse(input: Span) -> PResult<Self> {
        map(separated_list1(wst(lexem::DOT), parse_id), |value| {
            if value.len() == 1 {
                Variable::Var(value.first().unwrap().to_string())
            } else {
                Variable::FieldAccess(value)
            }
        })(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number(Number),
    Float(f64),
    Bool(bool),
    Char(char),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Slice {
    String(String),
    List(MultiData),
}

impl TryParse for Slice {
    /*
     * @desc Parse List or Static string
     *
     * @grammar
     * Slice := \[ MultData \] | StaticString
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                delimited(wst(lexem::SQ_BRA_O), MultiData::parse, wst(lexem::SQ_BRA_C)),
                |value| Slice::List(value),
            ),
            map(parse_string, |value| Slice::String(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vector {
    Init(MultiData),
    Def { length: usize, capacity: usize },
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
        alt((
            map(
                preceded(
                    wst(lexem::platform::VEC),
                    delimited(wst(lexem::SQ_BRA_O), MultiData::parse, wst(lexem::SQ_BRA_C)),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple(pub MultiData);

impl TryParse for Tuple {
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(wst(lexem::PAR_O), MultiData::parse, wst(lexem::PAR_C)),
            |value| Tuple(value),
        )(input)
    }
}

pub type MultiData = Vec<Expression>;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    params: Vec<ClosureParam>,
    scope: ClosureScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureScope {
    Scope(ast::statements::scope::Scope),
    Expr(Box<Expression>),
}

impl TryParse for Closure {
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), ClosureParam::parse),
                    wst(lexem::PAR_C),
                ),
                preceded(wst(lexem::ARROW), ClosureScope::parse),
            ),
            |(params, scope)| Closure { params, scope },
        )(input)
    }
}

impl TryParse for ClosureScope {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(ast::statements::scope::Scope::parse, |value| {
                ClosureScope::Scope(value)
            }),
            map(Expression::parse, |value| {
                ClosureScope::Expr(Box::new(value))
            }),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosureParam {
    Full { id: ID, signature: ast::types::Type },
    Minimal(ID),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Address(pub Box<Expression>);

impl TryParse for Address {
    /*
     * @desc Parse pointer address
     *
     * @grammar
     * Addr := &DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(preceded(wst(lexem::ADDR), Expression::parse), |value| {
            Address(Box::new(value))
        })(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Access(pub Box<Expression>);
impl TryParse for Access {
    /*
     * @desc Parse pointer access
     *
     * @grammar
     * Addr := -DATA
     */
    fn parse(input: Span) -> PResult<Self> {
        map(preceded(wst(lexem::ACCESS), Expression::parse), |value| {
            Access(Box::new(value))
        })(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Channel {
    Receive { addr: Address, timeout: usize },
    Send { addr: Address, msg: Box<Expression> },
    Init(String),
}

impl TryParse for Channel {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Struct {
    Inline {
        id: ID,
        data: MultiData,
    },
    Field {
        id: ID,
        fields: Vec<(String, Expression)>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Union {
    Inline {
        typename: ID,
        variant: ID,
        data: MultiData,
    },
    Field {
        typename: ID,
        variant: ID,
        fields: Vec<(String, Expression)>,
    },
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
        alt((
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(
                            wst(lexem::COMA),
                            separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                        ),
                        wst(lexem::BRA_C),
                    ),
                ),
                |(id, value)| Struct::Field { id, fields: value },
            ),
            map(
                pair(
                    parse_id,
                    delimited(wst(lexem::PAR_O), MultiData::parse, wst(lexem::PAR_C)),
                ),
                |(id, data)| Struct::Inline { id, data },
            ),
        ))(input)
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
        alt((
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(
                            wst(lexem::COMA),
                            separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                        ),
                        wst(lexem::BRA_C),
                    ),
                ),
                |((typename, name), value)| Union::Field {
                    variant: name,
                    typename: typename,
                    fields: value,
                },
            ),
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(wst(lexem::PAR_O), MultiData::parse, wst(lexem::PAR_C)),
                ),
                |((typename, name), data)| Union::Inline {
                    variant: name,
                    typename,
                    data,
                },
            ),
        ))(input)
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Enum {
    typename: ID,
    value: ID,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Map {
    Init { fields: Vec<(KeyData, Expression)> },
    Def { length: usize, capacity: usize },
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
                        wst(lexem::BRA_C),
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

#[derive(Debug, Clone, PartialEq)]
pub enum KeyData {
    Number(Number),
    Bool(bool),
    Char(char),
    String(String),
    Address(Address),
    Enum(Enum),
}

impl TryParse for KeyData {
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
            map(parse_string, |value| KeyData::String(value)),
            map(parse_number, |value| KeyData::Number(value)),
            map(
                alt((
                    value(true, wst(lexem::TRUE)),
                    value(false, wst(lexem::FALSE)),
                )),
                |value| KeyData::Bool(value),
            ),
            map(parse_char, |value| KeyData::Char(value)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::expressions::Atomic;

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
        let res = Slice::parse("[2,5,6]".into());
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

        let res = Slice::parse("\"Hello World\"".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Slice::String("Hello World".to_string()), value);
    }

    #[test]
    fn valid_vector() {
        let res = Vector::parse("vec[2,5,6]".into());
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

        let res = Vector::parse("vec(2,8)".into());
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
        let res = Closure::parse("(x,y) -> x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Closure {
                params: vec![
                    ClosureParam::Minimal("x".into()),
                    ClosureParam::Minimal("y".into())
                ],
                scope: ClosureScope::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Variable(Variable::Var("x".into()))
                ))))
            },
            value
        )
    }

    #[test]
    fn valid_chan() {
        let res = Channel::parse("receive[&chan1](10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Channel::Receive {
                addr: Address(Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                    Variable::Var("chan1".into())
                ))))),
                timeout: 10
            },
            value
        );

        let res = Channel::parse("send[&chan1](10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Channel::Send {
                addr: Address(Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                    Variable::Var("chan1".into())
                ))))),
                msg: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Number(10)
                ))))
            },
            value
        );

        let res = Channel::parse("chan(\"ID_CHAN\")".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Channel::Init("ID_CHAN".into()), value);
    }

    #[test]
    fn valid_tuple() {
        let res = Tuple::parse("(2,'a')".into());
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
        let res = Address::parse("&x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Address(Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                Variable::Var("x".into())
            ))))),
            value
        );
    }

    #[test]
    fn valid_access() {
        let res = Access::parse("*x".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Access(Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                Variable::Var("x".into())
            ))))),
            value
        );
    }

    #[test]
    fn valid_map() {
        let res = Map::parse(r#"map{"x":2,"y":6}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Map::Init {
                fields: vec![
                    (
                        KeyData::String("x".into()),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2))))
                    ),
                    (
                        KeyData::String("y".into()),
                        Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(6))))
                    )
                ]
            },
            value
        );

        let res = Map::parse(r#"map(2,8)"#.into());
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
        let res = Struct::parse(r#"Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Struct::Field {
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

        let res = Struct::parse(r#"Point (2, 8)"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Struct::Inline {
                id: "Point".into(),
                data: vec![
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2)))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(8))))
                ]
            },
            value
        )
    }

    #[test]
    fn valid_union() {
        let res = Union::parse(r#"Geo::Point { x : 2, y : 8}"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Union::Field {
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

        let res = Union::parse(r#"Geo::Point (2, 8)"#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Union::Inline {
                typename: "Geo".into(),
                variant: "Point".into(),
                data: vec![
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(2)))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(8))))
                ]
            },
            value
        )
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
}
