use nom::{
    branch::alt,
    combinator::map,
    sequence::{separated_pair, terminated},
};
use nom_supreme::ParserExt;

use crate::ast::{
    expressions::Expression,
    statements::block::Block,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
    TryParse,
};

use super::{AssignValue, Assignation};

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
            separated_pair(Expression::parse, wst(lexem::EQUAL), AssignValue::parse),
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

#[cfg(test)]
mod tests {

    use crate::{
        ast::expressions::{
            data::{Data, Variable},
            operation::FieldAccess,
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
                left: Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                    id: "x".to_string().into(),
                    from_field: false,
                    metadata: Metadata::default(),
                }))),
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
                left: Expression::Atomic(Atomic::Data(Data::PtrAccess(
                    crate::ast::expressions::data::PtrAccess {
                        value: Atomic::Data(Data::Variable(Variable {
                            id: "x".to_string().into(),
                            from_field: false,
                            metadata: Metadata::default(),
                        }))
                        .into(),
                        metadata: Metadata::default()
                    }
                ))),
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
                left: Expression::FieldAccess(FieldAccess {
                    var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "x".to_string().into(),
                        from_field: false,
                        metadata: Metadata::default(),
                    })))),
                    field: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "y".to_string().into(),
                        from_field: false,
                        metadata: Metadata::default(),
                    })))),
                    metadata: Metadata::default()
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(v_num!(Unresolved, 10))
                ))))
            },
            value
        );
    }
}
