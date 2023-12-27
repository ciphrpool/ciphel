use std::collections::{HashMap, HashSet};

use nom::{
    branch::alt,
    combinator::map,
    multi::{fold_many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use crate::parser::{
    ast::{
        types::{Type, Types},
        TryParse,
    },
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst, ID},
    },
};

use super::{declaration::TypedVar, scope::Scope};

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Struct(StructDef),
    Union(UnionDef),
    Enum(EnumDef),
    Fn(FnDef),
    Event(EventDef),
}

impl TryParse for Definition {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(StructDef::parse, |value| Definition::Struct(value)),
            map(UnionDef::parse, |value| Definition::Union(value)),
            map(EnumDef::parse, |value| Definition::Enum(value)),
            map(FnDef::parse, |value| Definition::Fn(value)),
            map(EventDef::parse, |value| Definition::Event(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    id: ID,
    fields: StructVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructVariant {
    Fields(Vec<(ID, Type)>),
    Inline(Types),
}

impl TryParse for StructVariant {
    /*
     * @desc Parse struct variant
     *
     * @grammar
     * { Struct_fields } | struct ID \( Types \)
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Type::parse),
                    ),
                    wst(lexem::BRA_C),
                ),
                |value| StructVariant::Fields(value),
            ),
            map(
                delimited(wst(lexem::PAR_O), Types::parse, wst(lexem::PAR_C)),
                |value| StructVariant::Inline(value),
            ),
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
            pair(preceded(wst(lexem::STRUCT), parse_id), StructVariant::parse),
            |(id, fields)| StructDef { id, fields },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionDef {
    id: ID,
    variants: Vec<(ID, UnionVariant)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnionVariant {
    Id,
    Fields(Vec<(ID, Type)>),
    Inline(Types),
}
impl TryParse for UnionVariant {
    /*
     * @desc Parse struct variant
     *
     * @grammar
     * Union_field := ID | ID { Struct_fields } | ID \(  Types \)
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        separated_pair(parse_id, wst(lexem::COLON), Type::parse),
                    ),
                    wst(lexem::BRA_C),
                ),
                |value| UnionVariant::Fields(value),
            ),
            map(
                delimited(wst(lexem::PAR_O), Types::parse, wst(lexem::PAR_C)),
                |value| UnionVariant::Inline(value),
            ),
        ))(input)
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
                preceded(wst(lexem::STRUCT), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(
                        wst(lexem::COMA),
                        alt((
                            map(parse_id, |value| (value, UnionVariant::Id)),
                            pair(parse_id, UnionVariant::parse),
                        )),
                    ),
                    wst(lexem::BRA_C),
                ),
            ),
            |(id, variants)| UnionDef { id, variants },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    id: ID,
    values: Vec<ID>,
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
                preceded(wst(lexem::STRUCT), parse_id),
                delimited(
                    wst(lexem::BRA_O),
                    separated_list1(wst(lexem::COMA), parse_id),
                    wst(lexem::BRA_C),
                ),
            ),
            |(id, values)| EnumDef { id, values },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    id: ID,
    params: Vec<TypedVar>,
    ret: Box<Type>,
    scope: Scope,
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
                separated_list0(wst(lexem::COMA), TypedVar::parse),
                preceded(wst(lexem::ARROW), Type::parse),
                Scope::parse,
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

#[derive(Debug, Clone, PartialEq)]
pub struct EventDef {
    id: ID,
    condition: EventCondition,
    scope: Scope,
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
                Scope::parse,
            )),
            |(id, condition, scope)| EventDef {
                id,
                condition,
                scope,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventCondition {}

impl TryParse for EventCondition {
    fn parse(input: Span) -> PResult<Self> {
        todo!()
    }
}
