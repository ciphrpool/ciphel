use std::sync::{Arc, RwLock};

use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, tuple},
};
use nom_supreme::ParserExt;

use crate::{
    ast::{
        expressions::{
            data::{Data, Primitive},
            Atomic, Expression,
        },
        statements::{
            self,
            assignation::Assignation,
            block::Block,
            declaration::{Declaration, PatternVar},
            Statement,
        },
        utils::{
            error::squash,
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst, wst_closed},
        },
        TryParse,
    },
    semantic::{scope::ClosureState, Metadata},
};

use super::{ForInit, ForInits, ForLoop, Loop, WhileLoop};

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
        squash(
            alt((
                map(preceded(wst_closed(lexem::LOOP), Block::parse), |scope| {
                    Loop::Loop(Box::new(scope))
                }),
                map(ForLoop::parse, |value| Loop::For(value)),
                //map(ForInLoop::parse, |value| Loop::ForIn(value)),
                map(WhileLoop::parse, |value| Loop::While(value)),
            )),
            "Expected a loop, while or for loop",
        )(input)
    }
}

impl TryParse for ForInit {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(Assignation::parse_without_semicolon, ForInit::Assignation),
            map(Declaration::parse_without_semicolon, ForInit::Declaration),
        ))(input)
    }
}
impl TryParse for ForLoop {
    fn parse(input: Span) -> PResult<Self> {
        map(
            tuple((
                wst_closed(lexem::FOR),
                delimited(
                    wst(lexem::PAR_O),
                    tuple((
                        separated_list0(wst(lexem::COMA), ForInit::parse),
                        wst(lexem::SEMI_COLON),
                        opt(Expression::parse),
                        wst(lexem::SEMI_COLON),
                        separated_list0(wst(lexem::COMA), Assignation::parse_without_semicolon),
                    )),
                    wst(lexem::PAR_C),
                ),
                cut(Block::parse).context("Invalid block in for-loop statement"),
            )),
            |(_, (indices, _, condition, _, increments), mut inner)| {
                let mut statements = Vec::new();
                for index in indices {
                    match index {
                        ForInit::Assignation(value) => {
                            statements.push(Statement::Assignation(value))
                        }
                        ForInit::Declaration(value) => {
                            statements.push(Statement::Declaration(value))
                        }
                    }
                }

                for increment in increments {
                    inner.statements.push(Statement::Assignation(increment));
                }

                statements.push(Statement::Loops(Loop::While(WhileLoop {
                    condition: Box::new(condition.unwrap_or(Expression::Atomic(Atomic::Data(
                        Data::Primitive(Primitive::Bool(true)),
                    )))),
                    block: Box::new(inner),
                })));

                let block = Block::new(statements);

                ForLoop {
                    block: Box::new(block),
                }
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
            pair(
                preceded(wst_closed(lexem::WHILE), cut(Expression::parse)),
                cut(Block::parse).context("Invalid block in while-loop statement"),
            ),
            |(condition, block)| WhileLoop {
                condition: Box::new(condition),
                block: Box::new(block),
            },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        ast::{
            expressions::{
                data::{Data, Primitive, Variable},
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
        for (let i:u8 = 0 ; i < 10 ; i = i + 1) {
            f(10);
        }
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
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
                block: Box::new(Block::new(vec![Statement::Flow(Flow::Call(CallStat {
                    call: Expression::FnCall(FnCall {
                        lib: None,
                        fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                name: "f".to_string().into(),
                                metadata: Metadata::default(),
                                state: None,
                            }
                        )))),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))],
                        metadata: Metadata::default(),
                        platform: Default::default(),
                        is_dynamic_fn: None,
                    })
                }))]))
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
            Loop::Loop(Box::new(Block::new(vec![Statement::Flow(Flow::Call(
                CallStat {
                    call: Expression::FnCall(FnCall {
                        lib: None,
                        fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
                            Variable {
                                name: "f".to_string().into(),
                                metadata: Metadata::default(),
                                state: None,
                            }
                        )))),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))],
                        metadata: Metadata::default(),
                        platform: Default::default(),
                        is_dynamic_fn: None,
                    })
                }
            ))]))),
            value
        );
    }
}
