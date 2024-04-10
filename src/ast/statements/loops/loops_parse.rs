use nom::{
    branch::alt,
    combinator::map,
    sequence::{delimited, pair, preceded, tuple},
};

use crate::{
    ast::{
        expressions::{
            data::{Address, Slice, Vector},
            Expression,
        },
        statements::{declaration::PatternVar, scope::Scope},
        utils::{
            io::{PResult, Span},
            lexem,
            numbers::parse_number,
            strings::{parse_id, wst},
        },
        TryParse,
    },
    semantic::scope::ScopeApi,
};

use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};

impl<InnerScope: ScopeApi> TryParse for Loop<InnerScope> {
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
            map(preceded(wst(lexem::LOOP), Scope::parse), |scope| {
                Loop::Loop(Box::new(scope))
            }),
            map(ForLoop::parse, |value| Loop::For(value)),
            map(WhileLoop::parse, |value| Loop::While(value)),
        ))(input)
    }
}

impl<Scope: ScopeApi> TryParse for ForIterator<Scope> {
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

impl<InnerScope: ScopeApi> TryParse for ForLoop<InnerScope> {
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
                Scope::parse,
            )),
            |(item, iterator, scope)| ForLoop {
                item,
                iterator,
                scope: Box::new(scope),
            },
        )(input)
    }
}

impl<InnerScope: ScopeApi> TryParse for WhileLoop<InnerScope> {
    /*
     * @desc Parse while loop
     *
     * @grammar
     *  while Expr Scope
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            pair(preceded(wst(lexem::WHILE), Expression::parse), Scope::parse),
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
    };

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, VarID, Variable},
                flows::FnCall,
                Atomic,
            },
            statements::{
                flows::{CallStat, Flow},
                Statement,
            },
        },
        semantic::{
            scope::{scope_impl::MockScope, ClosureState},
            Metadata,
        },
    };

    use super::*;

    #[test]
    fn valid_for() {
        let res = ForLoop::<MockScope>::parse(
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
                item: ForItem::Id("i".into()),
                iterator: ForIterator {
                    expr: Expression::Atomic(Atomic::Data(Data::Variable(Variable::Var(VarID {
                        id: "x".into(),
                        metadata: Metadata::default()
                    }))))
                },
                scope: Box::new(Scope {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Cell::new(Number::Unresolved(10)))
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        }
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None)
                })
            },
            value
        );
    }

    #[test]
    fn valid_while() {
        let res = WhileLoop::<MockScope>::parse(
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
                scope: Box::new(Scope {
                    metadata: Metadata::default(),
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID {
                                id: "f".into(),
                                metadata: Metadata::default()
                            }),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(Cell::new(Number::Unresolved(10)))
                            )))],
                            metadata: Metadata::default(),
                            platform: Rc::default(),
                        }
                    }))],
                    can_capture: Cell::new(ClosureState::DEFAULT),
                    is_loop: Cell::new(false),
                    is_generator: Cell::new(false),
                    caller: Default::default(),
                    inner_scope: RefCell::new(None)
                })
            },
            value
        );
    }

    #[test]
    fn valid_loop() {
        let res = Loop::<MockScope>::parse(
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
            Loop::Loop(Box::new(Scope {
                metadata: Metadata::default(),
                instructions: vec![Statement::Flow(Flow::Call(CallStat {
                    call: FnCall {
                        fn_var: Variable::Var(VarID {
                            id: "f".into(),
                            metadata: Metadata::default()
                        }),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(Cell::new(Number::Unresolved(10)))
                        )))],
                        metadata: Metadata::default(),
                        platform: Rc::default(),
                    }
                }))],
                can_capture: Cell::new(ClosureState::DEFAULT),
                is_loop: Cell::new(false),
                is_generator: Cell::new(false),
                caller: Default::default(),
                inner_scope: RefCell::new(None)
            })),
            value
        );
    }
}
