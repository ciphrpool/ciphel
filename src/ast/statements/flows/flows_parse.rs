use crate::ast::{
    expressions::operation::{operation_parse::TryParseOperation, FnCall},
    statements::block::Block,
    TryParse,
};
use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{many0, many1, separated_list1},
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

impl TryParse for Flow {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(IfStat::parse, |value| Flow::If(value)),
            map(MatchStat::parse, |value| Flow::Match(value)),
            map(TryStat::parse, |value| Flow::Try(value)),
            map(CallStat::parse, |value| Flow::Call(value)),
        ))(input)
    }
}

impl TryParse for IfStat {
    /*
     * @desc Parse If statement
     *
     * @grammar
     * If_Stat :=  if Expr Scope ( else Scope | Λ )
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                pair(preceded(wst(lexem::IF), Expression::parse), Block::parse),
                pair(
                    many0(pair(
                        preceded(pair(wst(lexem::ELSE), wst(lexem::IF)), Expression::parse),
                        Block::parse,
                    )),
                    opt(preceded(wst(lexem::ELSE), Block::parse)),
                ),
            ),
            |((condition, then_branch), (else_if_branches, else_branch))| IfStat {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_if_branches,
                else_branch: else_branch.map(|value| Box::new(value)),
            },
        )(input)
    }
}

impl TryParse for MatchStat {
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
                        many1(PatternStat::parse),
                        opt(preceded(
                            wst(lexem::ELSE),
                            preceded(wst(lexem::BIGARROW), Block::parse),
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

impl TryParse for PatternStat {
    /*
     * @desc Parse pattern statements
     *
     * @grammar
     * case Pattern => Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(
                    wst(lexem::CASE),
                    separated_list1(wst(lexem::BAR), Pattern::parse),
                ),
                wst(lexem::BIGARROW),
                Block::parse,
            ),
            |(patterns, scope)| PatternStat {
                patterns,
                scope: Box::new(scope),
            },
        )(input)
    }
}

impl TryParse for TryStat {
    /*
     * @desc Parse try statements
     *
     * @grammar
     * Try_Stat := try Scope ( else Scope | Λ  )
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::TRY), Block::parse),
                opt(preceded(wst(lexem::ELSE), Block::parse)),
            ),
            |(try_branch, else_branch)| TryStat {
                try_branch: Box::new(try_branch),
                else_branch: else_branch.map(|value| Box::new(value)),
            },
        )(input)
    }
}

impl TryParse for CallStat {
    /*
     * @desc Parse call statements
     *
     * @grammar
     * FnCallStat := ID \(  Fn_Args \) ;
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            terminated(FnCall::parse_statement, wst(lexem::SEMI_COLON)),
            |call| CallStat { call },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Variable},
                Atomic, Expression,
            },
            statements::{
                block::Block,
                flows::{CallStat, Flow, IfStat, TryStat},
                Statement,
            },
        },
        semantic::{scope::ClosureState, Metadata},
        v_num,
    };

    use super::*;

    #[test]
    fn valid_if() {
        let res = IfStat::parse(
            r#"
        if true {
            f(10);
        } else {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            IfStat {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                then_branch: Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }),
                else_if_branches: Vec::default(),
                else_branch: Some(Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }))
            },
            value
        );
    }

    #[test]
    fn valid_if_else_if() {
        let res = IfStat::parse(
            r#"
        if true {
            f(10);
        }
        else if true {
            f(10);
        } else {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            IfStat {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                then_branch: Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }),
                else_if_branches: vec![(
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Bool(true)))),
                    Block {
                        metadata: Metadata::default(),
                        instructions: vec![Statement::Flow(Flow::Call(CallStat {
                            call: Expression::FnCall(FnCall {
                                lib: None,
                                fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                    Variable {
                                        id: "f".to_string().into(),
                                        from_field: Cell::new(false),
                                        metadata: Metadata::default(),
                                    }
                                )))),
                                params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                    v_num!(Unresolved, 10)
                                )))],
                                metadata: Metadata::default(),
                                platform: Rc::default(),
                            })
                        }))],
                        can_capture: Cell::new(ClosureState::DEFAULT),
                        is_loop: Cell::new(false),

                        caller: Default::default(),
                        inner_scope: None,
                    }
                )],
                else_branch: Some(Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }))
            },
            value
        );
    }

    #[test]
    fn valid_try() {
        let res = TryStat::parse(
            r#"
        try {
            f(10);
        } else {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            TryStat {
                try_branch: Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }),
                else_branch: Some(Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: Cell::new(false),
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        })
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),

                    caller: Default::default(),
                    inner_scope: None,
                }))
            },
            value
        );
    }
}
