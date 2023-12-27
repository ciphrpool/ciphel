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
    condition: Box<Expression>,
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
                pair(
                    preceded(wst(lexem::IF), Expression::parse),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
                preceded(
                    wst(lexem::ELSE),
                    delimited(wst(lexem::BRA_O), Expression::parse, wst(lexem::BRA_C)),
                ),
            ),
            |((condition, main_branch), else_branch)| IfExpr {
                condition: Box::new(condition),
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
                separated_pair(parse_id, wst(lexem::SEP), parse_id),
                |(typename, value)| Pattern::Enum { typename, value },
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
                |(typename, vars)| Pattern::StructFields { typename, vars },
            ),
            map(
                delimited(
                    wst(lexem::PAR_O),
                    separated_list1(wst(lexem::COMA), parse_id),
                    wst(lexem::PAR_C),
                ),
                |value| Pattern::Tuple(value),
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

#[cfg(test)]
mod tests {
    use crate::parser::ast::expressions::data::Data;

    use super::*;

    #[test]
    fn valid_if() {
        let res = IfExpr::parse("if true { 10 } else { 20 }".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            IfExpr {
                condition: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true)))),
                main_branch: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                else_branch: Box::new(Expression::Data(Data::Primitive(Primitive::Number(20)))),
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
            case "Hello world" => true,
            case Geo::Point => true,
            case Geo::Point(y) => true,
            case Geo::Point{y} => true,
            case Point(y) => true,
            case Point{y} => true,
            case (y,z) => true,
            else => true
        }"#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            MatchExpr {
                expr: Box::new(Expression::Data(Data::Variable("x".into()))),
                patterns: vec![
                    PatternExpr {
                        pattern: Pattern::Primitive(Primitive::Number(10)),
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::String("Hello world".into()),
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::Enum {
                            typename: "Geo".into(),
                            value: "Point".into()
                        },
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::UnionInline {
                            typename: "Geo".into(),
                            variant: "Point".into(),
                            vars: vec!["y".into()]
                        },
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::UnionFields {
                            typename: "Geo".into(),
                            variant: "Point".into(),
                            vars: vec!["y".into()]
                        },
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::StructInline {
                            typename: "Point".into(),
                            vars: vec!["y".into()]
                        },
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::StructFields {
                            typename: "Point".into(),
                            vars: vec!["y".into()]
                        },
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    },
                    PatternExpr {
                        pattern: Pattern::Tuple(vec!["y".into(), "z".into()]),
                        expr: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
                    }
                ],
                else_branch: Box::new(Expression::Data(Data::Primitive(Primitive::Bool(true))))
            },
            value
        );
    }

    #[test]
    fn valid_try() {
        let res = TryExpr::parse("try { 10 } else { 20 }".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            TryExpr {
                try_branch: Box::new(Expression::Data(Data::Primitive(Primitive::Number(10)))),
                else_branch: Box::new(Expression::Data(Data::Primitive(Primitive::Number(20)))),
            },
            value
        );
    }

    #[test]
    fn valid_fn_call() {
        let res = FnCall::parse("f(x,10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(
            FnCall {
                fn_id: "f".into(),
                params: vec![
                    Expression::Data(Data::Variable("x".into())),
                    Expression::Data(Data::Primitive(Primitive::Number(10)))
                ]
            },
            value
        );
    }
}
