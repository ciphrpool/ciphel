use nom::{
    branch::alt,
    combinator::{map, value},
    multi::separated_list1,
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
    AddrType, ChanType, FnType, KeyType, MapType, NumberType, PrimitiveType, SliceType,
    StrSliceType, StringType, TupleType, Type, Types, VecType,
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
            map(StringType::parse, |value| Type::String(value)),
            value(Type::Unit, wst(lexem::UUNIT)),
            map(VecType::parse, |value| Type::Vec(value)),
            map(FnType::parse, |value| Type::Fn(value)),
            map(ChanType::parse, |value| Type::Chan(value)),
            map(TupleType::parse, |value| Type::Tuple(value)),
            map(AddrType::parse, |value| Type::Address(value)),
            map(MapType::parse, |value| Type::Map(value)),
            map(parse_id, |value| Type::UserType(value)),
        ))(input)
    }
}
impl TryParse for PrimitiveType {
    /*
     * @desc Parse Primitive types
     *
     * @grammar
     * Primitive := num | float | char | bool
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(NumberType::parse, |num| PrimitiveType::Number(num)),
            value(PrimitiveType::Char, wst(lexem::CHAR)),
            value(PrimitiveType::Bool, wst(lexem::BOOL)),
        ))(input)
    }
}

impl TryParse for NumberType {
    /*
     * @desc Parse Primitive types
     *
     * @grammar
     * num := u8 | u16 | u32 | u64 | u128 | i8 | i16 | i32 | i64 | i128
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            value(NumberType::U8, wst(lexem::U8)),
            value(NumberType::U16, wst(lexem::U16)),
            value(NumberType::U32, wst(lexem::U32)),
            value(NumberType::U64, wst(lexem::U64)),
            value(NumberType::U128, wst(lexem::U128)),
            value(NumberType::I8, wst(lexem::I8)),
            value(NumberType::I16, wst(lexem::I16)),
            value(NumberType::I32, wst(lexem::I32)),
            value(NumberType::I64, wst(lexem::I64)),
            value(NumberType::I128, wst(lexem::I128)),
            value(NumberType::F64, wst(lexem::FLOAT)),
        ))(input)
    }
}

impl TryParse for SliceType {
    /*
     * @desc Parse Slice types
     *
     * @grammar
     * Slice :=
     *  | [ num ] Type
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                delimited(wst(lexem::SQ_BRA_O), parse_number, wst(lexem::SQ_BRA_C)),
                Type::parse,
            ),
            |(size, value)| SliceType {
                size: size as usize,
                item_type: Box::new(value),
            },
        )(input)
    }
}

impl TryParse for StrSliceType {
    /*
     * @desc Parse Slice types
     *
     * @grammar
     * Slice :=
     *  | [ num ] str
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                delimited(wst(lexem::SQ_BRA_O), parse_number, wst(lexem::SQ_BRA_C)),
                wst(lexem::STR),
            ),
            |(size, value)| StrSliceType {
                size: size as usize,
            },
        )(input)
    }
}

impl TryParse for StringType {
    /*
     * @desc Parse Slice types
     *
     * @grammar
     * String := string
     */
    fn parse(input: Span) -> PResult<Self> {
        value(StringType(), wst(lexem::STRING))(input)
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
            map(StringType::parse, |value| KeyType::String(value)),
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
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Bool, value);

        let res = PrimitiveType::parse("u8".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Number(NumberType::U8), value);

        let res = PrimitiveType::parse("char".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Char, value);

        let res = PrimitiveType::parse("f64".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(PrimitiveType::Number(NumberType::F64), value);
    }

    #[test]
    fn valid_string() {
        let res = StringType::parse("string".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(StringType(), value);
    }

    #[test]
    fn valid_slice_type() {
        let res = SliceType::parse("[8]u32".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            SliceType {
                size: 8,
                item_type: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::U32)))
            },
            value
        );

        let res = SliceType::parse("[8][2]i32".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            SliceType {
                size: 8,
                item_type: Box::new(Type::Slice(SliceType {
                    size: 2,
                    item_type: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::I32)))
                }))
            },
            value
        );
    }

    #[test]
    fn valid_vec_type() {
        let res = VecType::parse("Vec<[8]u128>".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            VecType(Box::new(Type::Slice(SliceType {
                size: 8,
                item_type: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::U128)))
            }))),
            value
        );

        let res = VecType::parse("Vec<u64>".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            VecType(Box::new(Type::Primitive(PrimitiveType::Number(
                NumberType::U64
            )))),
            value
        );
    }

    #[test]
    fn valid_fn_type() {
        let res = FnType::parse("fn(u16) -> bool".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            FnType {
                params: vec![Type::Primitive(PrimitiveType::Number(NumberType::U16))],
                ret: Box::new(Type::Primitive(PrimitiveType::Bool))
            },
            value
        );
    }

    #[test]
    fn valid_chan_type() {
        let res = ChanType::parse("Chan<[8]i128>".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            ChanType(Box::new(Type::Slice(SliceType {
                size: 8,
                item_type: Box::new(Type::Primitive(PrimitiveType::Number(NumberType::I128)))
            }))),
            value
        );
    }

    #[test]
    fn valid_address_type() {
        let res = AddrType::parse("&u64".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            AddrType(Box::new(Type::Primitive(PrimitiveType::Number(
                NumberType::U64
            )))),
            value
        );
    }

    #[test]
    fn valid_map_type() {
        let res = MapType::parse("Map<string,bool>".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            MapType {
                keys_type: KeyType::String(StringType()),
                values_type: Box::new(Type::Primitive(PrimitiveType::Bool))
            },
            value
        );
    }

    #[test]
    fn valid_tuple_type() {
        let res = TupleType::parse("(i32,u16)".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            TupleType(vec![
                Type::Primitive(PrimitiveType::Number(NumberType::I32)),
                Type::Primitive(PrimitiveType::Number(NumberType::U16))
            ]),
            value
        );
    }
}
