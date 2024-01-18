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
        alt((
            map(parse_id, |value| ForIterator::Id(value)),
            map(Vector::parse, |value| ForIterator::Vec(value)),
            map(Slice::parse, |value| ForIterator::Slice(value)),
            map(
                tuple((
                    wst(lexem::platform::RECEIVE),
                    delimited(wst(lexem::SQ_BRA_O), Address::parse, wst(lexem::SQ_BRA_C)),
                    delimited(wst(lexem::PAR_O), parse_number, wst(lexem::PAR_C)),
                )),
                |(_, addr, timeout)| ForIterator::Receive {
                    addr,
                    timeout: timeout.unsigned_abs() as usize,
                },
            ),
        ))(input)
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
    use std::cell::RefCell;

    use crate::{
        ast::{
            expressions::{
                data::{Data, Primitive, VarID, Variable},
                flows::FnCall,
                Atomic,
            },
            statements::{
                flows::{CallStat, Flow},
                Statement,
            },
        },
        semantic::scope::scope_impl::MockScope,
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
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            ForLoop {
                item: ForItem::Id("i".into()),
                iterator: ForIterator::Id("x".into()),
                scope: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID("f".into())),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(10)
                            )))]
                        }
                    }))],

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
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            WhileLoop {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                scope: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        call: FnCall {
                            fn_var: Variable::Var(VarID("f".into())),
                            params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                                Primitive::Number(10)
                            )))]
                        }
                    }))],

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
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            Loop::Loop(Box::new(Scope {
                instructions: vec![Statement::Flow(Flow::Call(CallStat {
                    call: FnCall {
                        fn_var: Variable::Var(VarID("f".into())),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(10)
                        )))]
                    }
                }))],

                inner_scope: RefCell::new(None)
            })),
            value
        );
    }
}
