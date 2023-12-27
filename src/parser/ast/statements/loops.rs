use nom::{
    branch::alt,
    combinator::map,
    sequence::{delimited, pair, preceded, tuple},
};

use crate::parser::{
    ast::{
        expressions::{
            data::{Address, Slice, Tuple, Vector},
            Expression,
        },
        TryParse,
    },
    utils::{
        io::{PResult, Span},
        lexem,
        numbers::parse_number,
        strings::{parse_id, wst, ID},
    },
};

use super::{declaration::DeclaredVar, scope::Scope};

#[derive(Debug, Clone, PartialEq)]
pub enum Loop {
    For(ForLoop),
    While(WhileLoop),
    Loop(Box<Scope>),
}

impl TryParse for Loop {
    /*
     * @desc Parse loop
     *
     * @grammar
     * Loop :=
     *      | for ITEM in ITERATOR Scope
     *      | while Expr Scope
     *      | loop Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(preceded(wst(lexem::LOOP), Scope::parse), |scope| {
                Loop::Loop(Box::new(scope))
            }),
            map(ForLoop::parse, |value| Loop::For(value)),
            map(WhileLoop::parse, |value| Loop::While(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    item: DeclaredVar,
    iterator: ForIterator,
    scope: Box<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForIterator {
    Id(ID),
    Vec(Vector),
    Slice(Slice),
    Tuple(Tuple),
    Receive { addr: Address, timeout: usize },
}

impl TryParse for ForIterator {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(parse_id, |value| ForIterator::Id(value)),
            map(Vector::parse, |value| ForIterator::Vec(value)),
            map(Slice::parse, |value| ForIterator::Slice(value)),
            map(Tuple::parse, |value| ForIterator::Tuple(value)),
            map(
                tuple((
                    wst(lexem::platform::RECEIVE),
                    delimited(wst(lexem::SQ_BRA_O), Address::parse, wst(lexem::SQ_BRA_C)),
                    delimited(wst(lexem::PAR_O), parse_number, wst(lexem::PAR_C)),
                )),
                |(_, addr, timeout)| ForIterator::Receive {
                    addr,
                    timeout: timeout.unsigned_abs() as usize,
                },
            ),
        ))(input)
    }
}

impl TryParse for ForLoop {
    /*
     * @desc Parse for loop
     *
     * @grammar
     * Forloop := for ITEM in ITERATOR Scope
     * ITERATOR := ID | VecData | SliceData | receive\[ Addr \]\(  UNumber \) | TupleData
     * ITEM := ID | TypedVar | PatternVar
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                preceded(wst(lexem::FOR), DeclaredVar::parse),
                preceded(wst(lexem::IN), ForIterator::parse),
                Scope::parse,
            )),
            |(item, iterator, scope)| ForLoop {
                item,
                iterator,
                scope: Box::new(scope),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    condition: Box<Expression>,
    scope: Box<Scope>,
}

impl TryParse for WhileLoop {
    /*
     * @desc Parse while loop
     *
     * @grammar
     *  while Expr Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(preceded(wst(lexem::WHILE), Expression::parse), Scope::parse),
            |(condition, scope)| WhileLoop {
                condition: Box::new(condition),
                scope: Box::new(scope),
            },
        )(input)
    }
}
