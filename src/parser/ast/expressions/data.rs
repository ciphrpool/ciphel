use std::collections::HashMap;

use nom::{
    branch::alt,
    combinator::{map, value},
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
            parse_id,
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
    Unit,
    Map(Map),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Variable(ID),
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
            map(parse_id, |value| Data::Variable(value)),
        ))(input)
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
pub struct Tuple(MultiData);

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
    scope: ast::statements::scope::Scope,
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
                ast::statements::scope::Scope::parse,
            ),
            |(params, scope)| Closure { params, scope },
        )(input)
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
pub struct Address(Box<Expression>);

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
pub struct Access(Box<Expression>);
impl TryParse for Access {
    /*
     * @desc Parse pointer access
     *
     * @grammar
     * Addr := &DATA
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
        fields: HashMap<String, Expression>,
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
        fields: HashMap<String, Expression>,
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
                        fold_many0(
                            separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                            HashMap::new,
                            |mut acc, (key, value)| {
                                acc.insert(key, value);
                                acc
                            },
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
                        fold_many0(
                            separated_pair(parse_id, wst(lexem::COLON), Expression::parse),
                            HashMap::new,
                            |mut acc, (key, value)| {
                                acc.insert(key, value);
                                acc
                            },
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
                        fold_many0(
                            separated_pair(KeyData::parse, wst(lexem::COLON), Expression::parse),
                            Vec::new,
                            |mut acc, (key, value)| {
                                acc.push((key, value));
                                acc
                            },
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
    use super::*;

    #[test]
    fn valid_primitive_data() {
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
                Expression::Data(Data::Primitive(Primitive::Number(2))),
                Expression::Data(Data::Primitive(Primitive::Number(5))),
                Expression::Data(Data::Primitive(Primitive::Number(6)))
            ]),
            value
        );

        let res = Slice::parse("\"Hello World\"".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Slice::String("Hello World".to_string()), value);
    }
}
