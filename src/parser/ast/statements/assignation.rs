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

#[cfg(test)]
mod tests {
    use crate::parser::ast::expressions::{
        data::{Data, Primitive, Variable},
        Atomic,
    };

    use super::*;

    #[test]
    fn valid_assignation() {
        let res = Assignation::parse("x = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::Variable("x".into()),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );

        let res = Assignation::parse("*x = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::PointerAccess(Access(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Variable(Variable::Var("x".into()))
                ))))),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );

        let res = Assignation::parse("x.y = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::FieldAccess(vec!["x".into(), "y".into()]),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );
    }
}
