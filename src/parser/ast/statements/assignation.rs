use nom::{
    branch::alt,
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, terminated},
};

use crate::parser::{
    ast::{
        expressions::{data::Access, Expression},
        TryParse,
    },
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst, ID},
    },
};

use super::scope::Scope;

#[derive(Debug, Clone, PartialEq)]

pub struct Assignation {
    left: Assignee,
    right: AssignValue,
}

impl TryParse for Assignation {
    /*
     * @desc Parse assignation
     *
     * @grammar
     * Assignation :=
     *      | Access = Scope | Expr ;
     *      | * Expr = Scope | Expr ;
     * Access := ID . Access | ID
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(Assignee::parse, wst(lexem::EQUAL), AssignValue::parse),
            |(left, right)| Assignation { left, right },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignValue {
    Scope(Scope),
    Expr(Box<Expression>),
}

impl TryParse for AssignValue {
    /*
     * @desc Parse assigned value
     *
     * @grammar
     * Scope | Expr ;
     */
    fn parse(input: Span) -> PResult<Self> {
        terminated(
            alt((
                map(Scope::parse, |value| AssignValue::Scope(value)),
                map(Expression::parse, |value| {
                    AssignValue::Expr(Box::new(value))
                }),
            )),
            wst(lexem::SEMI_COLON),
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Assignee {
    Variable(ID),
    FieldAccess(Vec<ID>),
    PointerAccess(Access),
}

impl TryParse for Assignee {
    /*
     * @desc Parse assigne
     *
     * @grammar
     * Assigne := ID | Access | * Expr
     * Access := ID . Access | ID
     *
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Access::parse, |value| Assignee::PointerAccess(value)),
            map(separated_list1(wst(lexem::DOT), parse_id), |value| {
                if value.len() == 1 {
                    Assignee::Variable(value.first().unwrap().to_string())
                } else {
                    Assignee::FieldAccess(value)
                }
            }),
        ))(input)
    }
}
