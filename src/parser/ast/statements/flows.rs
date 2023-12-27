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
    condition: Box<Expression>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CallStat {
    pub fn_id: ID,
    pub params: Vec<Expression>,
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

#[cfg(test)]
mod tests {
    use crate::parser::ast::{
        expressions::{
            data::{Data, Primitive},
            Atomic,
        },
        statements::Statement,
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
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            IfStat {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                main_branch: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        fn_id: "f".into(),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(10)
                        )))]
                    }))]
                }),
                else_branch: Some(Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        fn_id: "f".into(),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(10)
                        )))]
                    }))]
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
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            TryStat {
                try_branch: Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        fn_id: "f".into(),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(10)
                        )))]
                    }))]
                }),
                else_branch: Some(Box::new(Scope {
                    instructions: vec![Statement::Flow(Flow::Call(CallStat {
                        fn_id: "f".into(),
                        params: vec![Expression::Atomic(Atomic::Data(Data::Primitive(
                            Primitive::Number(10)
                        )))]
                    }))]
                }))
            },
            value
        );
    }

    #[test]
    fn valid_return() {
        let res = Return::parse(
            r#"
            return ;
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Return::Unit, value);
    }
}
