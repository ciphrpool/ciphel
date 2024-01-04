use nom::{
    branch::alt,
    combinator::{map, value},
    multi::{separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::ast::utils::{
    io::{PResult, Span},
    lexem,
    numbers::parse_number,
    strings::{parse_id, wst},
};

use crate::ast::TryParse;

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};

impl TryParse for Type {
    /*
     * @desc Parse Type
     *
     * @grammar
     * Type :=  Primitive | ID | Vec |  Fn | Chan | Slice | Tuple | Unit | Address | Map
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(PrimitiveType::parse, |value| Type::Primitive(value)),
            map(SliceType::parse, |value| Type::Slice(value)),
            map(parse_id, |value| Type::UserType(value)),
            map(VecType::parse, |value| Type::Vec(value)),
            map(FnType::parse, |value| Type::Fn(value)),
            map(ChanType::parse, |value| Type::Chan(value)),
            map(TupleType::parse, |value| Type::Tuple(value)),
            value(Type::Unit, wst(lexem::UUNIT)),
            map(AddrType::parse, |value| Type::Address(value)),
            map(MapType::parse, |value| Type::Map(value)),
        ))(input)
    }
}
impl TryParse for PrimitiveType {
    /*
     * @desc Parse Primitive types
     *
     * @grammar
     * Primitive := num | int | float | char | bool
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(PrimitiveType::Number, wst(lexem::NUMBER)),
            value(PrimitiveType::Float, wst(lexem::FLOAT)),
            value(PrimitiveType::Char, wst(lexem::CHAR)),
            value(PrimitiveType::Bool, wst(lexem::BOOL)),
        ))(input)
    }
}
impl TryParse for SliceType {
    /*
     * @desc Parse Slice types
     *
     * @grammar
     * Slice :=
     *  | string
     *  | [ num ] Type
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(SliceType::String, wst(lexem::STRING)),
            map(
                pair(
                    delimited(wst(lexem::SQ_BRA_O), parse_number, wst(lexem::SQ_BRA_C)),
                    Type::parse,
                ),
                |(size, value)| SliceType::List(size.unsigned_abs() as usize, Box::new(value)),
            ),
        ))(input)
    }
}

impl TryParse for VecType {
    /*
     * @desc Parse Vec Type
     *
     * @grammar
     * Vec := Vec< Type >
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::UVEC),
                delimited(wst(lexem::LESSER), Type::parse, wst(lexem::GREATER)),
            ),
            |value| VecType(Box::new(value)),
        )(input)
    }
}

impl TryParse for FnType {
    /*
     * @desc Parse Closure Type Definition
     *
     * @grammar
     * Fn := Fn(Types) -> Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                wst(lexem::FN),
                delimited(wst(lexem::PAR_O), Types::parse, wst(lexem::PAR_C)),
                wst(lexem::ARROW),
                Type::parse,
            )),
            |(_, params, _, ret)| FnType {
                params,
                ret: Box::new(ret),
            },
        )(input)
    }
}

impl TryParse for Types {
    /*
     * @desc Parse multiple Types
     *
     * @grammar
     * Types := Type , Types
     */
    fn parse(input: Span) -> PResult<Self> {
        separated_list1(wst(lexem::COMA), Type::parse)(input)
    }
}

impl TryParse for ChanType {
    /*
     * @desc Parse Chan Type
     *
     * @grammar
     * Chan := Chan< Type >
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::UCHAN),
                delimited(wst(lexem::LESSER), Type::parse, wst(lexem::GREATER)),
            ),
            |value| ChanType(Box::new(value)),
        )(input)
    }
}

impl TryParse for TupleType {
    /*
     * @desc Parse Tuple Type
     *
     * @grammar
     * Tuple := \(  Types \)
     * Types := Type , Types | Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(wst(lexem::PAR_O), Types::parse, wst(lexem::PAR_C)),
            |params| TupleType(params),
        )(input)
    }
}

impl TryParse for AddrType {
    /*
     * @desc Parse Address Type
     *
     * @grammar
     * Address := & Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(preceded(wst(lexem::ADDR), Type::parse), |value| {
            AddrType(Box::new(value))
        })(input)
    }
}

impl TryParse for MapType {
    /*
     * @desc Parse Map Type
     *
     * @grammar
     * Map := map < Key , Types >
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::UMAP),
                delimited(
                    wst(lexem::LESSER),
                    separated_pair(KeyType::parse, wst(lexem::COMA), Type::parse),
                    wst(lexem::GREATER),
                ),
            ),
            |(keys, values)| MapType {
                keys_type: keys,
                values_type: Box::new(values),
            },
        )(input)
    }
}

impl TryParse for KeyType {
    /*
     * @desc Parse Key Type
     *
     * @grammar
     * Key := Primitive | Address | Slice
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(PrimitiveType::parse, |value| KeyType::Primitive(value)),
            map(SliceType::parse, |value| KeyType::Slice(value)),
            map(AddrType::parse, |value| KeyType::Address(value)),
            map(parse_id, |value| KeyType::EnumID(value)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_primitive_type() {
        let res = PrimitiveType::parse("bool".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Bool, value);

        let res = PrimitiveType::parse("number".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Number, value);

        let res = PrimitiveType::parse("char".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Char, value);

        let res = PrimitiveType::parse("float".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Float, value);
    }

    #[test]
    fn valid_slice_type() {
        let res = SliceType::parse("string".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(SliceType::String, value);

        let res = SliceType::parse("[8]number".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            SliceType::List(8, Box::new(Type::Primitive(PrimitiveType::Number))),
            value
        );

        let res = SliceType::parse("[8][2]number".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            SliceType::List(
                8,
                Box::new(Type::Slice(SliceType::List(
                    2,
                    Box::new(Type::Primitive(PrimitiveType::Number))
                )))
            ),
            value
        );
    }

    #[test]
    fn valid_vec_type() {
        let res = VecType::parse("Vec<[8]number>".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            VecType(Box::new(Type::Slice(SliceType::List(
                8,
                Box::new(Type::Primitive(PrimitiveType::Number))
            )))),
            value
        );
    }

    #[test]
    fn valid_fn_type() {
        let res = FnType::parse("fn(number) -> bool".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            FnType {
                params: vec![Type::Primitive(PrimitiveType::Number)],
                ret: Box::new(Type::Primitive(PrimitiveType::Bool))
            },
            value
        );
    }

    #[test]
    fn valid_chan_type() {
        let res = ChanType::parse("Chan<[8]number>".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            ChanType(Box::new(Type::Slice(SliceType::List(
                8,
                Box::new(Type::Primitive(PrimitiveType::Number))
            )))),
            value
        );
    }

    #[test]
    fn valid_address_type() {
        let res = AddrType::parse("&number".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            AddrType(Box::new(Type::Primitive(PrimitiveType::Number))),
            value
        );
    }

    #[test]
    fn valid_map_type() {
        let res = MapType::parse("Map<string,bool>".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            MapType {
                keys_type: KeyType::Slice(SliceType::String),
                values_type: Box::new(Type::Primitive(PrimitiveType::Bool))
            },
            value
        );
    }

    #[test]
    fn valid_tuple_type() {
        let res = TupleType::parse("(number,number)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            TupleType(vec![
                Type::Primitive(PrimitiveType::Number),
                Type::Primitive(PrimitiveType::Number)
            ]),
            value
        );
    }
}
