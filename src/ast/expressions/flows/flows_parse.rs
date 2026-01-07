use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair},
};

use crate::{
    ast::{
        expressions::{
            data::{Primitive, StrSlice},
            CompletePath, Expression,
        },
        statements::block::{BlockCommonApi, ExprBlock},
        types::Type,
        utils::{
            error::squash,
            io::{PResult, Span},
            lexem,
            strings::{parse_id, wst, wst_closed},
        },
        TryParse,
    },
    semantic::{Metadata, Resolve},
    vm::GenerateCode,
};
use std::fmt::Debug;

use super::{
    Cases, EnumCase, ExprFlow, IfExpr, MatchExpr, PrimitiveCase, StringCase, TryExpr, UnionCase,
    UnionPattern,
};

impl TryParse for ExprFlow {
    /*
     * @desc Parse expression statement
     *
     * @grammar
     * ExprStatement := If_Expr | Match_Expr | Try_Expr | Call
     */
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(IfExpr::parse, |value| ExprFlow::If(value)),
            map(MatchExpr::parse, |value| ExprFlow::Match(value)),
            map(TryExpr::parse, |value| ExprFlow::Try(value)),
            map(
                preceded(
                    wst(crate::vm::core::lexem::SIZEOF),
                    delimited(wst(lexem::PAR_O), Type::parse, wst(lexem::PAR_C)),
                ),
                |value| ExprFlow::SizeOf(value, Metadata::default()),
            ),
        ))(input)
    }
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
                    delimited(
                        wst_closed(lexem::IF),
                        Expression::parse,
                        wst_closed(lexem::THEN),
                    ),
                    cut(ExprBlock::parse),
                ),
                cut(preceded(wst_closed(lexem::ELSE), ExprBlock::parse)),
            ),
            |((condition, then_branch), else_branch)| IfExpr {
                condition: Box::new(condition),
                then_branch,
                else_branch,
                metadata: Metadata::default(),
            },
        )(input)
    }
}

impl TryParse for UnionPattern {
    /*
     * @desc Parse pattern
     *
     * @grammar
     * Pattern :=
     *      | ID :: ID { IDs }
     */
    fn parse(input: Span) -> PResult<Self> {
        squash(
            map(
                pair(
                    CompletePath::parse_segment,
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |(CompletePath { mut path, name }, vars_names)| {
                    let typename = match &mut path {
                        crate::ast::expressions::Path::Segment(vec) => match vec.pop() {
                            Some(typename) => {
                                if vec.len() == 0 {
                                    path = crate::ast::expressions::Path::Empty;
                                    typename
                                } else {
                                    typename
                                }
                            }
                            None => "".to_string(), // unreachable
                        },
                        crate::ast::expressions::Path::Empty => "".to_string(), // unreachable
                    };
                    UnionPattern {
                        path,
                        variant: name,
                        typename,
                        vars_names,
                        vars_id: None,
                        variant_value: None,
                        variant_padding: None,
                    }
                },
            ),
            "Expected a valid pattern",
        )(input)
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> TryParse
    for PrimitiveCase<B>
{
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(
                    wst_closed(lexem::CASE),
                    separated_list1(wst(lexem::BAR), Primitive::parse),
                ),
                wst(lexem::BIGARROW),
                B::parse,
            ),
            |(patterns, block)| PrimitiveCase { patterns, block },
        )(input)
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> TryParse
    for StringCase<B>
{
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(
                    wst_closed(lexem::CASE),
                    separated_list1(wst(lexem::BAR), StrSlice::parse),
                ),
                wst(lexem::BIGARROW),
                B::parse,
            ),
            |(patterns, block)| StringCase { patterns, block },
        )(input)
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> TryParse
    for EnumCase<B>
{
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(
                    wst_closed(lexem::CASE),
                    separated_list1(
                        wst(lexem::BAR),
                        map(crate::ast::expressions::data::Enum::parse, |enum_variant| {
                            (enum_variant, None)
                        }),
                    ),
                ),
                wst(lexem::BIGARROW),
                B::parse,
            ),
            |(patterns, block)| EnumCase { patterns, block },
        )(input)
    }
}

impl<B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq> TryParse
    for UnionCase<B>
{
    fn parse(input: Span) -> PResult<Self> {
        map(
            separated_pair(
                preceded(wst_closed(lexem::CASE), UnionPattern::parse),
                wst(lexem::BIGARROW),
                B::parse,
            ),
            |(pattern, block)| UnionCase { pattern, block },
        )(input)
    }
}

impl<
        B: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
        C: TryParse + Resolve + GenerateCode + BlockCommonApi + Clone + Debug + PartialEq,
    > TryParse for Cases<B, C>
{
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(
                separated_list1(wst(lexem::COMA), UnionCase::<C>::parse),
                |cases| Cases::Union { cases },
            ),
            map(
                separated_list1(wst(lexem::COMA), PrimitiveCase::<B>::parse),
                |cases| Cases::Primitive { cases },
            ),
            map(
                separated_list1(wst(lexem::COMA), StringCase::<B>::parse),
                |cases| Cases::String { cases },
            ),
            map(
                separated_list1(wst(lexem::COMA), EnumCase::<B>::parse),
                |cases| Cases::Enum { cases },
            ),
        ))(input)
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
                preceded(wst_closed(lexem::MATCH), cut(Expression::parse)),
                cut(delimited(
                    wst(lexem::BRA_O),
                    pair(
                        Cases::parse,
                        opt(preceded(
                            wst(lexem::COMA),
                            preceded(
                                wst_closed(lexem::ELSE),
                                preceded(wst(lexem::BIGARROW), ExprBlock::parse),
                            ),
                        )),
                    ),
                    preceded(opt(wst(lexem::COMA)), wst(lexem::BRA_C)),
                )),
            ),
            |(expr, (cases, else_branch))| MatchExpr {
                expr: Box::new(expr),
                cases,
                else_branch,
                metadata: Metadata::default(),
            },
        )(input)
    }
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
                preceded(wst_closed(lexem::TRY), ExprBlock::parse),
                opt(preceded(wst_closed(lexem::ELSE), ExprBlock::parse)),
            ),
            |(try_branch, else_branch)| TryExpr {
                try_branch,
                else_branch,
                pop_last_err: false,
                metadata: Metadata::default(),
            },
        )(input)
    }
}
