use crate::ast::{
    expressions::{
        data::{Call, Printf},
        flows::Cases,
        operation::operation_parse::TryParseOperation,
    },
    statements::block::Block,
    utils::{error::squash, strings::wst_closed},
    TryParse,
};
use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};
use nom_supreme::ParserExt;

use crate::ast::{
    expressions::Expression,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
};

use super::{CallStat, Flow, IfStat, MatchStat, TryStat};

impl TryParse for Flow {
    fn parse(input: Span) -> PResult<Self> {
        squash(
            alt((
                map(IfStat::parse, |value| Flow::If(value)),
                map(MatchStat::parse, |value| Flow::Match(value)),
                map(TryStat::parse, |value| Flow::Try(value)),
                map(terminated(Printf::parse, wst(lexem::SEMI_COLON)), |value| {
                    Flow::Printf(value)
                }),
                map(CallStat::parse, |value| Flow::Call(value)),
            )),
            "Expected an if, match, try statement or a function call",
        )(input)
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
                pair(
                    preceded(
                        wst_closed(lexem::IF),
                        cut(Expression::parse).context("Invalid condition"),
                    ),
                    cut(Block::parse).context("Invalid block in if statement"),
                ),
                pair(
                    many0(pair(
                        preceded(
                            pair(wst_closed(lexem::ELSE), wst_closed(lexem::IF)),
                            cut(Expression::parse).context("Invalid condition"),
                        ),
                        cut(Block::parse).context("Invalid block in else if statement"),
                    )),
                    opt(preceded(
                        wst_closed(lexem::ELSE),
                        cut(Block::parse).context("Invalid block in else statement"),
                    )),
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
                preceded(wst_closed(lexem::MATCH), cut(Expression::parse)),
                delimited(
                    cut(wst(lexem::BRA_O)),
                    cut(pair(
                        Cases::parse,
                        opt(preceded(
                            preceded(opt(wst(lexem::COMA)), wst_closed(lexem::ELSE)),
                            preceded(
                                wst(lexem::BIGARROW),
                                cut(Block::parse).context("Invalid block in else statement"),
                            ),
                        )),
                    )),
                    cut(preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C))),
                ),
            ),
            |(expr, (cases, else_branch))| MatchStat {
                expr: Box::new(expr),
                cases,
                else_branch: else_branch.map(|value| Box::new(value)),
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
                preceded(
                    wst_closed(lexem::TRY),
                    cut(Block::parse).context("Invalid block in try statement"),
                ),
                opt(preceded(
                    wst_closed(lexem::ELSE),
                    cut(Block::parse).context("Invalid block in else statement"),
                )),
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
        map(terminated(Call::parse, wst(lexem::SEMI_COLON)), |call| {
            CallStat { call }
        })(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::statements::flows::{IfStat, TryStat};

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
    }
}
