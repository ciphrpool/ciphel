use nom::{
    branch::alt,
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use crate::ast::{
    expressions::Expression,
    statements::{block::Block, declaration::PatternVar},
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst},
    },
    TryParse,
};

use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};

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
            map(preceded(wst(lexem::LOOP), Block::parse), |scope| {
                Loop::Loop(Box::new(scope))
            }),
            map(ForLoop::parse, |value| Loop::For(value)),
            map(WhileLoop::parse, |value| Loop::While(value)),
        ))(input)
    }
}

impl TryParse for ForIterator {
    fn parse(input: Span) -> PResult<Self> {
        map(Expression::parse, |expr| ForIterator { expr })(input)
    }
}

impl TryParse for ForItem {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(PatternVar::parse, |value| ForItem::Pattern(value)),
            map(parse_id, |value| ForItem::Id(value)),
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
                preceded(wst(lexem::FOR), ForItem::parse),
                preceded(wst(lexem::IN), ForIterator::parse),
                Block::parse,
            )),
            |(item, iterator, scope)| ForLoop {
                item,
                iterator,
                scope: Box::new(scope),
            },
        )(input)
    }
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
            pair(preceded(wst(lexem::WHILE), Expression::parse), Block::parse),
            |(condition, scope)| WhileLoop {
                condition: Box::new(condition),
                scope: Box::new(scope),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::{Arc, RwLock},
    };

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, Variable},
                operation::FnCall,
                Atomic,
            },
            statements::{
                flows::{CallStat, Flow},
                Statement,
            },
        },
        semantic::{scope::ClosureState, Metadata},
        v_num,
    };

    use super::*;

    #[test]
    fn valid_for() {
        let res = ForLoop::parse(
            r#"
        for i in x {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            ForLoop {
                item: ForItem::Id("i".to_string().into()),
                iterator: ForIterator {
                    expr: Expression::Atomic(Atomic::Data(Data::Variable(Variable {
                        id: "x".to_string().into(),
                        from_field: false,
                        metadata: Metadata::default(),
                    })))
                },
                scope: Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: false,
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Default::default(),
                        })
                    }))],
                    can_capture: Arc::new(RwLock::new(ClosureState::DEFAULT)),
                    is_loop: Default::default(),

                    caller: Default::default(),
                    inner_scope: None
                })
            },
            value
        );
    }

    #[test]
    fn valid_while() {
        let res = WhileLoop::parse(
            r#"
        while true {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            WhileLoop {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                scope: Box::new(Block {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: Expression::FnCall(FnCall {
                            lib: None,
                            fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                                Variable {
                                    id: "f".to_string().into(),
                                    from_field: false,
                                    metadata: Metadata::default(),
                                }
                            )))),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                v_num!(Unresolved, 10)
                            )))],
                            metadata: Metadata::default(),
                            platform: Default::default(),
                        })
                    }))],
                    can_capture: Arc::new(RwLock::new(ClosureState::DEFAULT)),
                    is_loop: Default::default(),

                    caller: Default::default(),
                    inner_scope: None
                })
            },
            value
        );
    }

    #[test]
    fn valid_loop() {
        let res = Loop::parse(
            r#"
        loop {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            Loop::Loop(Box::new(Block {
                metadata: Metadata::default(),
                instructions: vec![Statement::Flow(Flow::Call(CallStat {
                    call: Expression::FnCall(FnCall {
                        lib: None,
                        fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                id: "f".to_string().into(),
                                from_field: false,
                                metadata: Metadata::default(),
                            }
                        )))),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))],
                        metadata: Metadata::default(),
                        platform: Default::default(),
                    })
                }))],
                can_capture: Arc::new(RwLock::new(ClosureState::DEFAULT)),
                is_loop: Default::default(),

                caller: Default::default(),
                inner_scope: None
            })),
            value
        );
    }
}
