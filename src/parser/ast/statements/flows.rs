use nom::{
    branch::alt,
    combinator::{map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::parser::{
    ast::{
        expressions::{flows::Pattern, Expression},
        TryParse,
    },
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, wst, ID},
    },
};

use super::scope::Scope;

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    If(IfStat),
    Match(MatchStat),
    Try(TryStat),
    Call(CallStat),
    Return(Return),
}

impl TryParse for Flow {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(IfStat::parse, |value| Flow::If(value)),
            map(MatchStat::parse, |value| Flow::Match(value)),
            map(TryStat::parse, |value| Flow::Try(value)),
            map(CallStat::parse, |value| Flow::Call(value)),
            map(Return::parse, |value| Flow::Return(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStat {
    main_branch: Box<Scope>,
    else_branch: Option<Box<Scope>>,
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
                preceded(
                    wst(lexem::IF),
                    delimited(wst(lexem::BRA_O), Scope::parse, wst(lexem::BRA_C)),
                ),
                opt(preceded(
                    wst(lexem::ELSE),
                    delimited(wst(lexem::BRA_O), Scope::parse, wst(lexem::BRA_C)),
                )),
            ),
            |(main_branch, else_branch)| IfStat {
                main_branch: Box::new(main_branch),
                else_branch: else_branch.map(|value| Box::new(value)),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStat {
    expr: Box<Expression>,
    patterns: Vec<PatternStat>,
    else_branch: Option<Box<Scope>>,
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
                        separated_list1(wst(lexem::COMA), PatternStat::parse),
                        opt(preceded(
                            wst(lexem::COMA),
                            preceded(
                                wst(lexem::ELSE),
                                preceded(wst(lexem::BIGARROW), Scope::parse),
                            ),
                        )),
                    ),
                    wst(lexem::BRA_C),
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

#[derive(Debug, Clone, PartialEq)]
pub struct PatternStat {
    pattern: Pattern,
    scope: Box<Scope>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct TryStat {
    try_branch: Box<Scope>,
    else_branch: Option<Box<Scope>>,
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
                    wst(lexem::TRY),
                    delimited(wst(lexem::BRA_O), Scope::parse, wst(lexem::BRA_C)),
                ),
                opt(preceded(
                    wst(lexem::ELSE),
                    delimited(wst(lexem::BRA_O), Scope::parse, wst(lexem::BRA_C)),
                )),
            ),
            |(try_branch, else_branch)| TryStat {
                try_branch: Box::new(try_branch),
                else_branch: else_branch.map(|value| Box::new(value)),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallStat {
    fn_id: ID,
    params: Vec<Expression>,
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
            terminated(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list0(wst(lexem::COMA), Expression::parse),
                        wst(lexem::PAR_C),
                    ),
                ),
                wst(lexem::SEMI_COLON),
            ),
            |(fn_id, params)| CallStat { fn_id, params },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Unit,
    Expr(Box<Expression>),
}

impl TryParse for Return {
    /*
     * @desc Parse return statements
     *
     * @grammar
     * Return := return
     *      | ID
     *      | Expr
     *      | Λ
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::RETURN),
                opt(Expression::parse),
                wst(lexem::SEMI_COLON),
            ),
            |value| match value {
                Some(expr) => Return::Expr(Box::new(expr)),
                None => Return::Unit,
            },
        )(input)
    }
}
