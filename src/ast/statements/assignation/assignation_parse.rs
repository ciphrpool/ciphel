use nom::{
    branch::alt,
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, terminated},
};

use crate::{
    ast::{
        expressions::{
            data::{PtrAccess, Variable},
            Expression,
        },
        statements::scope::Scope,
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst, ID},
        },
        TryParse,
    },
    semantic::{scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError},
};

use super::{AssignValue, Assignation, Assignee};

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
            map(PtrAccess::parse, |value| Assignee::PtrAccess(value)),
            map(Variable::parse, |value| Assignee::Variable(value)),
        ))(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expressions::{
        data::{Data, FieldAccess, Primitive, VarID, Variable},
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
                left: Assignee::Variable(Variable::Var(VarID("x".into()))),
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
                left: Assignee::PtrAccess(PtrAccess(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Variable(Variable::Var(VarID("x".into())))
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
                left: Assignee::Variable(Variable::FieldAccess(FieldAccess {
                    var: Box::new(Variable::Var(VarID("x".into()))),
                    field: Box::new(Variable::Var(VarID("y".into())))
                })),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(10))
                ))))
            },
            value
        );
    }
}
