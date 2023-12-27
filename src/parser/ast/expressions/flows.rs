use nom::{
    branch::alt,
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
};

use crate::parser::{
    ast::TryParse,
    utils::{
        io::{PResult, Span},
        lexem,
        strings::{parse_id, string_parser::parse_string, wst, ID},
    },
};

use super::{data::Primitive, Expression};

#[derive(Debug, Clone, PartialEq)]
pub enum ExprFlow {
    If(IfExpr),
    Match(MatchExpr),
    Try(TryExpr),
    Call(FnCall),
}

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
            map(FnCall::parse, |value| ExprFlow::Call(value)),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    main_branch: Box<Expression>,
    else_branch: Box<Expression>,
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
                preceded(
                    wst(lexem::IF),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
                preceded(
                    wst(lexem::ELSE),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
            ),
            |(main_branch, else_branch)| IfExpr {
                main_branch: Box::new(main_branch),
                else_branch: Box::new(else_branch),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    expr: Box<Expression>,
    patterns: Vec<PatternExpr>,
    else_branch: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Primitive(Primitive),
    String(String),
    Enum {
        typename: ID,
        value: ID,
    },
    UnionInline {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    UnionFields {
        typename: ID,
        variant: ID,
        vars: Vec<ID>,
    },
    StructInline {
        typename: ID,
        vars: Vec<ID>,
    },
    StructFields {
        typename: ID,
        vars: Vec<ID>,
    },
    Tuple(Vec<ID>),
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
            map(parse_string, |value| Pattern::String(value)),
            map(
                separated_pair(parse_id, wst(lexem::SEP), parse_id),
                |(typename, value)| Pattern::Enum { typename, value },
            ),
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::PAR_C),
                    ),
                ),
                |((typename, variant), vars)| Pattern::UnionInline {
                    typename,
                    variant,
                    vars,
                },
            ),
            map(
                pair(
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |((typename, variant), vars)| Pattern::UnionFields {
                    typename,
                    variant,
                    vars,
                },
            ),
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::PAR_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::PAR_C),
                    ),
                ),
                |(typename, vars)| Pattern::StructInline { typename, vars },
            ),
            map(
                pair(
                    parse_id,
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |(typename, vars)| Pattern::StructInline { typename, vars },
            ),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternExpr {
    pattern: Pattern,
    expr: Box<Expression>,
}

impl TryParse for PatternExpr {
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(wst(lexem::CASE), Pattern::parse),
                wst(lexem::BIGARROW),
                Expression::parse,
            ),
            |(pattern, expr)| PatternExpr {
                pattern,
                expr: Box::new(expr),
            },
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
                        preceded(
                            wst(lexem::COMA),
                            preceded(
                                wst(lexem::ELSE),
                                preceded(wst(lexem::BIGARROW), Expression::parse),
                            ),
                        ),
                    ),
                    wst(lexem::BRA_C),
                ),
            ),
            |(expr, (patterns, else_branch))| MatchExpr {
                expr: Box::new(expr),
                patterns,
                else_branch: Box::new(else_branch),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryExpr {
    try_branch: Box<Expression>,
    else_branch: Box<Expression>,
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
                preceded(
                    wst(lexem::TRY),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
                preceded(
                    wst(lexem::ELSE),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
            ),
            |(try_branch, else_branch)| TryExpr {
                try_branch: Box::new(try_branch),
                else_branch: Box::new(else_branch),
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall {
    fn_id: ID,
    params: Vec<Expression>,
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
                parse_id,
                delimited(
                    wst(lexem::PAR_O),
                    separated_list0(wst(lexem::COMA), Expression::parse),
                    wst(lexem::PAR_C),
                ),
            ),
            |(fn_id, params)| FnCall { fn_id, params },
        )(input)
    }
}
