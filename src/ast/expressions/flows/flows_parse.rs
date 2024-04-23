use std::rc::Rc;

use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::{
    ast::{
        expressions::{
            data::{ExprScope, Primitive, StrSlice, Variable},
            Expression,
        },
        types::Type,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::{parse_id, string_parser::parse_string, wst},
        },
        TryParse,
    },
    semantic::Metadata,
    vm,
};

use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};

impl TryParse for ExprFlow {
    /*
     * @desc Parse expression statement
     *
     * @grammar
     * ExprStatement := If_Expr | Match_Expr | Try_Expr | FnCall
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(IfExpr::parse, |value| ExprFlow::If(value)),
            map(MatchExpr::parse, |value| ExprFlow::Match(value)),
            map(TryExpr::parse, |value| ExprFlow::Try(value)),
            map(
                preceded(
                    wst(vm::platform::utils::lexem::SIZEOF),
                    delimited(wst(lexem::PAR_O), Type::parse, wst(lexem::PAR_C)),
                ),
                |value| ExprFlow::SizeOf(value, Metadata::default()),
            ),
            map(FnCall::parse, |value| ExprFlow::Call(value)),
        ))(input)
    }
}

impl TryParse for IfExpr {
    /*
     * @desc Parse If expression
     *
     * @grammar
     * If_Expr := if Expr { Expr} else { Expr }
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                pair(
                    delimited(wst(lexem::IF), Expression::parse, wst(lexem::THEN)),
                    ExprScope::parse,
                ),
                preceded(wst(lexem::ELSE), ExprScope::parse),
            ),
            |((condition, then_branch), else_branch)| IfExpr {
                condition: Box::new(condition),
                then_branch,
                else_branch,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for Pattern {
    /*
     * @desc Parse pattern
     *
     * @grammar
     * Pattern :=
     *      | PrimitiveData
     *      | String
     *      | ID :: ID
     *      | ID :: ID \( IDs \)
     *      | ID :: ID { IDs }
     *      | ID { IDs }
     *      | ID \( IDs \)
     *      | \(  IDs \)
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Primitive::parse, |value| Pattern::Primitive(value)),
            map(StrSlice::parse, |value| Pattern::String(value)),
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |((typename, variant), vars)| Pattern::Union {
                    typename,
                    variant,
                    vars,
                },
            ),
            map(
                separated_pair(parse_id, wst(lexem::SEP), parse_id),
                |(typename, value)| Pattern::Enum { typename, value },
            ),
            // map(
            //     pair(
            //         parse_id,
            //         delimited(
            //             wst(lexem::BRA_O),
            //             separated_list1(wst(lexem::COMA), parse_id),
            //             wst(lexem::BRA_C),
            //         ),
            //     ),
            //     |(typename, vars)| Pattern::Struct { typename, vars },
            // ),
            // map(
            //     delimited(
            //         wst(lexem::PAR_O),
            //         separated_list1(wst(lexem::COMA), parse_id),
            //         wst(lexem::PAR_C),
            //     ),
            //     |value| Pattern::Tuple(value),
            // ),
        ))(input)
    }
}

impl TryParse for PatternExpr {
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(wst(lexem::CASE), Pattern::parse),
                wst(lexem::BIGARROW),
                ExprScope::parse,
            ),
            |(pattern, expr)| PatternExpr { pattern, expr },
        )(input)
    }
}

impl TryParse for MatchExpr {
    /*
     * @desc Parse Match expression
     *
     * @grammar
     * Match_Expr := match expr { Patterns , else => Expr }
     * Patterns := case Pattern => Expr , Patterns | case Pattern => Expr
     * Pattern :=
     *      | PrimitiveData
     *      | String
     *      | ID :: ID
     *      | ID :: ID \( IDs \)
     *      | ID :: ID { IDs }
     *      | ID { IDs }
     *      | ID \( IDs \)
     *      | \(  IDs \)
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::MATCH), Expression::parse),
                delimited(
                    wst(lexem::BRA_O),
                    pair(
                        separated_list1(wst(lexem::COMA), PatternExpr::parse),
                        opt(preceded(
                            wst(lexem::COMA),
                            preceded(
                                wst(lexem::ELSE),
                                preceded(wst(lexem::BIGARROW), ExprScope::parse),
                            ),
                        )),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                ),
            ),
            |(expr, (patterns, else_branch))| MatchExpr {
                expr: Box::new(expr),
                patterns,
                else_branch,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for TryExpr {
    /*
     * @desc Parse try expression
     *
     * @grammar
     * Try_Expr := try { Expr } else { Expr}
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                preceded(wst(lexem::TRY), ExprScope::parse),
                preceded(wst(lexem::ELSE), ExprScope::parse),
            ),
            |(try_branch, else_branch)| TryExpr {
                try_branch,
                else_branch,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for FnCall {
    /*
     * @desc Parse fn call
     *
     * @grammar
     * FnCall := ID\( Fn_Args \)
     * Fn_Args := Expr , Fn_Args
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(
                pair(opt(terminated(parse_id, wst(lexem::SEP))), Variable::parse),
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), Expression::parse),
                    wst(lexem::PAR_C),
                ),
            ),
            |((lib, fn_var), params)| FnCall {
                lib,
                fn_var,
                params,
                metadata: Metadata::default(),
                platform: Rc::default(),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, StrSlice, VarID, Variable},
                flows::PatternExpr,
                Atomic, Expression,
            },
            statements::{block::Block, return_stat::Return, Statement},
        },
        semantic::{scope::ClosureState, Metadata},
        v_num,
    };

    use super::*;

    #[test]
    fn valid_if() {
        let res = IfExpr::parse("if true then 10 else 20".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            IfExpr {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                then_branch: ExprScope::Expr(Block {
                    metadata: Metadata::default(),
                    instructions: vec![
                        (Statement::Return(Return::Expr {
                            expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))),
                            metadata: Metadata::default()
                        }))
                    ],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                }),
                else_branch: ExprScope::Expr(Block {
                    metadata: Metadata::default(),
                    instructions: vec![
                        (Statement::Return(Return::Expr {
                            expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 20)
                            )))),
                            metadata: Metadata::default()
                        }))
                    ],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                }),
                metadata: Metadata::default(),
            },
            value
        );
    }

    #[test]
    fn valid_match() {
        let res = MatchExpr::parse(
            r#"
        match x { 
            case 10 => true,
            case "Hello World" => true,
            case Geo::Point => true,
            case Geo::Point{y} => true,
            // case Point{y} => true,
            // case (y,z) => true,
            else => true
        }"#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            MatchExpr {
                expr: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                    Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })
                )))),
                metadata: Metadata::default(),
                patterns: vec![
                    PatternExpr {
                        pattern: Pattern::Primitive(Primitive::Number(Cell::new(
                            Number::Unresolved(10)
                        ))),
                        expr: ExprScope::Expr(Block {
                            metadata: Metadata::default(),
                            instructions: vec![
                                (Statement::Return(Return::Expr {
                                    expr: Box::new(Expression::Atomic(Atomic::Data(
                                        Data::Primitive(Primitive::Bool(true))
                                    ))),
                                    metadata: Metadata::default()
                                }))
                            ],
                            can_capture: Cell::new(ClosureState::DEFAULT),
                            is_loop: Cell::new(false),
                            is_generator: Cell::new(false),
                            caller: Default::default(),
                            inner_scope: RefCell::new(None),
                        })
                    },
                    PatternExpr {
                        pattern: Pattern::String(StrSlice {
                            value: "Hello World".to_string(),
                            metadata: Metadata::default()
                        }),
                        expr: ExprScope::Expr(Block {
                            metadata: Metadata::default(),
                            instructions: vec![
                                (Statement::Return(Return::Expr {
                                    expr: Box::new(Expression::Atomic(Atomic::Data(
                                        Data::Primitive(Primitive::Bool(true))
                                    ))),
                                    metadata: Metadata::default()
                                }))
                            ],
                            can_capture: Cell::new(ClosureState::DEFAULT),
                            is_loop: Cell::new(false),
                            is_generator: Cell::new(false),
                            caller: Default::default(),
                            inner_scope: RefCell::new(None),
                        })
                    },
                    PatternExpr {
                        pattern: Pattern::Enum {
                            typename: "Geo".into(),
                            value: "Point".into()
                        },
                        expr: ExprScope::Expr(Block {
                            metadata: Metadata::default(),
                            instructions: vec![
                                (Statement::Return(Return::Expr {
                                    expr: Box::new(Expression::Atomic(Atomic::Data(
                                        Data::Primitive(Primitive::Bool(true))
                                    ))),
                                    metadata: Metadata::default()
                                }))
                            ],
                            can_capture: Cell::new(ClosureState::DEFAULT),
                            is_loop: Cell::new(false),
                            is_generator: Cell::new(false),
                            caller: Default::default(),
                            inner_scope: RefCell::new(None),
                        })
                    },
                    PatternExpr {
                        pattern: Pattern::Union {
                            typename: "Geo".into(),
                            variant: "Point".into(),
                            vars: vec!["y".into()]
                        },
                        expr: ExprScope::Expr(Block {
                            metadata: Metadata::default(),
                            instructions: vec![
                                (Statement::Return(Return::Expr {
                                    expr: Box::new(Expression::Atomic(Atomic::Data(
                                        Data::Primitive(Primitive::Bool(true))
                                    ))),
                                    metadata: Metadata::default()
                                }))
                            ],
                            can_capture: Cell::new(ClosureState::DEFAULT),
                            is_loop: Cell::new(false),
                            is_generator: Cell::new(false),
                            caller: Default::default(),
                            inner_scope: RefCell::new(None),
                        })
                    },
                    // PatternExpr {
                    //     pattern: Pattern::Struct {
                    //         typename: "Point".into(),
                    //         vars: vec!["y".into()]
                    //     },
                    //     expr: ExprScope::Expr(Scope {
                    //         metadata: Metadata::default(),
                    //         instructions: vec![
                    //             (Statement::Return(Return::Expr {
                    //                 expr: Box::new(Expression::Atomic(Atomic::Data(
                    //                     Data::Primitive(Primitive::Bool(true))
                    //                 ))),
                    //                 metadata: Metadata::default()
                    //             }))
                    //         ],

                    //         inner_scope: RefCell::new(None),
                    //     })
                    // },
                    // PatternExpr {
                    //     pattern: Pattern::Tuple(vec!["y".into(), "z".into()]),
                    //     expr: ExprScope::Expr(Scope {
                    //         metadata: Metadata::default(),
                    //         instructions: vec![
                    //             (Statement::Return(Return::Expr {
                    //                 expr: Box::new(Expression::Atomic(Atomic::Data(
                    //                     Data::Primitive(Primitive::Bool(true))
                    //                 ))),
                    //                 metadata: Metadata::default()
                    //             }))
                    //         ],

                    //         inner_scope: RefCell::new(None),
                    //     })
                    // }
                ],
                else_branch: Some(ExprScope::Expr(Block {
                    metadata: Metadata::default(),
                    instructions: vec![
                        (Statement::Return(Return::Expr {
                            expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Bool(true)
                            )))),
                            metadata: Metadata::default()
                        }))
                    ],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                }))
            },
            value
        );
    }

    #[test]
    fn valid_try() {
        let res = TryExpr::parse("try 10 else 20".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            TryExpr {
                try_branch: ExprScope::Expr(Block {
                    metadata: Metadata::default(),
                    instructions: vec![
                        (Statement::Return(Return::Expr {
                            expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))),
                            metadata: Metadata::default()
                        }))
                    ],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                }),
                else_branch: ExprScope::Expr(Block {
                    metadata: Metadata::default(),
                    instructions: vec![
                        (Statement::Return(Return::Expr {
                            expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 20)
                            )))),
                            metadata: Metadata::default()
                        }))
                    ],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None),
                }),
                metadata: Metadata::default(),
            },
            value
        );
    }

    #[test]
    fn valid_fn_call() {
        let res = FnCall::parse("f(x,10)".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            FnCall {
                lib: None,
                fn_var: Variable::Var(VarID {
                    id: "f".into(),
                    metadata: Metadata::default()
                }),
                params: vec![
                    Expression::Atomic(Atomic::Data(Data::Variable(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(10).into()
                    ))))
                ],
                metadata: Metadata::default(),
                platform: Rc::default(),
            },
            value
        );
    }

    #[test]
    fn valid_lib_call() {
        let res = FnCall::parse("core::f(x,10)".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            FnCall {
                lib: Some("core".into()),
                fn_var: Variable::Var(VarID {
                    id: "f".into(),
                    metadata: Metadata::default()
                }),
                params: vec![
                    Expression::Atomic(Atomic::Data(Data::Variable(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    })))),
                    Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(
                        Number::Unresolved(10).into()
                    ))))
                ],
                metadata: Metadata::default(),
                platform: Rc::default(),
            },
            value
        );
    }
}
