use crate::{
    ast::{expressions::flows::FnCall, statements::scope::Scope, TryParse},
    semantic::scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
};
use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::ast::{
    expressions::{flows::Pattern, Expression},
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, TryStat};

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TryParse for Flow<Scope>
{
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(IfStat::parse, |value| Flow::If(value)),
            map(MatchStat::parse, |value| Flow::Match(value)),
            map(TryStat::parse, |value| Flow::Try(value)),
            map(CallStat::parse, |value| Flow::Call(value)),
        ))(input)
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
    > TryParse for IfStat<InnerScope>
{
    /*
     * @desc Parse If statement
     *
     * @grammar
     * If_Stat :=  if Expr Scope ( else Scope | Λ )
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                pair(preceded(wst(lexem::IF), Expression::parse), Scope::parse),
                opt(preceded(wst(lexem::ELSE), Scope::parse)),
            ),
            |((condition, main_branch), else_branch)| IfStat {
                condition: Box::new(condition),
                main_branch: Box::new(main_branch),
                else_branch: else_branch.map(|value| Box::new(value)),
            },
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
    > TryParse for MatchStat<InnerScope>
{
    /*
     * @desc Parse match statements
     *
     * @grammar
     * Match_Stat := match expr { PatternsStat, else Scope | Λ )
     * PatternsStat := case Pattern => Scope PatternsStat | case Pattern => Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::MATCH), Expression::parse),
                delimited(
                    wst(lexem::BRA_O),
                    pair(
                        separated_list1(wst(lexem::COMA), PatternStat::parse),
                        opt(preceded(
                            wst(lexem::COMA),
                            preceded(
                                wst(lexem::ELSE),
                                preceded(wst(lexem::BIGARROW), Scope::parse),
                            ),
                        )),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(expr, (patterns, else_branch))| MatchStat {
                expr: Box::new(expr),
                patterns,
                else_branch: else_branch.map(|value| Box::new(value)),
            },
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
    > TryParse for PatternStat<InnerScope>
{
    /*
     * @desc Parse pattern statements
     *
     * @grammar
     * case Pattern => Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(wst(lexem::CASE), Pattern::parse),
                wst(lexem::BIGARROW),
                Scope::parse,
            ),
            |(pattern, scope)| PatternStat {
                pattern,
                scope: Box::new(scope),
            },
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
    > TryParse for TryStat<InnerScope>
{
    /*
     * @desc Parse try statements
     *
     * @grammar
     * Try_Stat := try Scope ( else Scope | Λ  )
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::TRY), Scope::parse),
                opt(preceded(wst(lexem::ELSE), Scope::parse)),
            ),
            |(try_branch, else_branch)| TryStat {
                try_branch: Box::new(try_branch),
                else_branch: else_branch.map(|value| Box::new(value)),
            },
        )(input)
    }
}

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > TryParse for CallStat<Scope>
{
    /*
     * @desc Parse call statements
     *
     * @grammar
     * FnCallStat := ID \(  Fn_Args \) ;
     */
    fn parse(input: Span) -> PResult<Self> {
        map(terminated(FnCall::parse, wst(lexem::SEMI_COLON)), |call| {
            CallStat { call }
        })(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, VarID, Variable},
                Atomic, Expression,
            },
            statements::{
                flows::{CallStat, Flow, IfStat, TryStat},
                scope::Scope,
                Statement,
            },
        },
        semantic::{scope::scope_impl::MockScope, Metadata},
    };

    use super::*;

    #[test]
    fn valid_if() {
        let res = IfStat::<MockScope>::parse(
            r#"
        if true {
            f(10);
        } else {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            IfStat {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                main_branch: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Number::U64(10))
                            )))]
                        }
                    }))],

                    inner_scope: RefCell::new(None),
                }),
                else_branch: Some(Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Number::U64(10))
                            )))]
                        }
                    }))],

                    inner_scope: RefCell::new(None),
                }))
            },
            value
        );
    }

    #[test]
    fn valid_try() {
        let res = TryStat::<MockScope>::parse(
            r#"
        try {
            f(10);
        } else {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            TryStat {
                try_branch: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Number::U64(10))
                            )))]
                        }
                    }))],

                    inner_scope: RefCell::new(None),
                }),
                else_branch: Some(Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Number::U64(10))
                            )))]
                        }
                    }))],

                    inner_scope: RefCell::new(None),
                }))
            },
            value
        );
    }
}
