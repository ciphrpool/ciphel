use nom::{
    branch::alt,
    combinator::map,
    sequence::{separated_pair, terminated},
};

use crate::{
    ast::{
        expressions::{
            data::{PtrAccess, Variable},
            Expression,
        },
        statements::scope::Scope,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    semantic::scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
};

use super::{AssignValue, Assignation, Assignee};

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TryParse for Assignation<Scope>
{
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

impl<
        InnerScope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TryParse for AssignValue<InnerScope>
{
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

impl<
        InnerScope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TryParse for Assignee<InnerScope>
{
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
    use crate::{
        ast::expressions::{
            data::{Data, FieldAccess, Number, Primitive, VarID, Variable},
            Atomic,
        },
        semantic::{scope::scope_impl::MockScope, Metadata},
    };

    use super::*;

    #[test]
    fn valid_assignation() {
        let res = Assignation::<MockScope>::parse("x = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::Variable(Variable::Var(VarID {
                    id: "x".into(),
                    metadata: Metadata::default()
                })),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(Number::U64(10)))
                ))))
            },
            value
        );

        let res = Assignation::<MockScope>::parse("*x = 10;".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Assignation {
                left: Assignee::PtrAccess(PtrAccess {
                    value: Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    }),
                    metadata: Metadata::default()
                }),
                right: AssignValue::Expr(Box::new(Expression::Atomic(Atomic::Data(
                    Data::Primitive(Primitive::Number(Number::U64(10)))
                ))))
            },
            value
        );

        let res = Assignation::<MockScope>::parse("x.y = 10;".into());
        assert!(res.is_ok());
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
                    Data::Primitive(Primitive::Number(Number::U64(10)))
                ))))
            },
            value
        );
    }
}
