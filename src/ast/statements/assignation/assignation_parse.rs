use nom::{
    branch::alt,
    combinator::map,
    sequence::{separated_pair, terminated},
};
use nom_supreme::ParserExt;

use crate::ast::{
    expressions::{
        data::{PtrAccess, Variable},
        Expression,
    },
    statements::block::Block,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
    TryParse,
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
                map(Block::parse, |value| AssignValue::Scope(value)),
                map(Expression::parse, |value| {
                    AssignValue::Expr(Box::new(value))
                }),
            )),
            wst(lexem::SEMI_COLON).context("expected a ;"),
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
    use std::cell::Cell;

    use crate::{
        ast::expressions::{
            data::{Data, FieldAccess, Number, Primitive, VarID, Variable},
            Atomic,
        },
        semantic::Metadata,
        v_num,
    };

    use super::*;

    #[test]
    fn valid_assignation() {
        let res = Assignation::parse("x = 10;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::Variable(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );

        let res = Assignation::parse("*x = 10;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::PtrAccess(PtrAccess {
                    value: Atomic::Data(Data::Variable(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })))
                    .into(),
                    metadata: Metadata::default()
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );

        let res = Assignation::parse("x.y = 10;".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::Variable(Variable::FieldAccess(FieldAccess {
                    var: Box::new(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })),
                    field: Box::new(Variable::Var(VarID {
                        id: "y".into(),
                        metadata: Metadata::default()
                    })),
                    metadata: Metadata::default()
                })),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );
    }
}
