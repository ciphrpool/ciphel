use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, tuple},
};
use nom_supreme::ParserExt;

use crate::ast::{
    expressions::Expression,
    statements::{assignation::Assignation, block::Block, declaration::Declaration},
    utils::{
        error::squash,
        io::{PResult, Span},
        lexem,
        strings::{wst, wst_closed},
    },
    TryParse,
};

use super::{ForInit, ForLoop, Loop, WhileLoop};

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
            |(_, (indices, _, condition, _, increments), block)| ForLoop {
                indices,
                condition,
                increments,
                block: Box::new(block),
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
    }
}
