use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::{many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
};

use crate::{
    ast::{
        expressions::{
            data::{Data, Primitive, StrSlice, Variable},
            Atomic, Expression,
        },
        statements::block::{BlockCommonApi, ExprBlock},
        types::Type,
        utils::{
            error::squash,
            io::{PResult, Span},
            lexem,
            strings::{parse_id, string_parser::parse_fstring, wst, wst_closed},
        },
        TryParse,
    },
    semantic::{Metadata, Resolve},
    vm::{self, core, vm::GenerateCode},
};
use std::fmt::Debug;

use super::{
    Cases, EnumCase, ExprFlow, FCall, FormatItem, IfExpr, MatchExpr, PrimitiveCase, StringCase,
    TryExpr, UnionCase, UnionPattern,
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
                    wst(vm::core::lexem::SIZEOF),
                    delimited(wst(lexem::PAR_O), Type::parse, wst(lexem::PAR_C)),
                ),
                |value| ExprFlow::SizeOf(value, Metadata::default()),
            ),
            // map(FCall::parse, |value| ExprFlow::FCall(value)),
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
                    separated_pair(parse_id, wst(lexem::SEP), parse_id),
                    delimited(
                        wst(lexem::BRA_O),
                        separated_list1(wst(lexem::COMA), parse_id),
                        wst(lexem::BRA_C),
                    ),
                ),
                |((typename, variant), vars_names)| UnionPattern {
                    typename,
                    variant,
                    vars_names,
                    vars_id: None,
                    variant_value: None,
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
                        map(
                            separated_pair(parse_id, wst(lexem::SEP), parse_id),
                            |(t, n)| (t, n, None),
                        ),
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

impl TryParse for FCall {
    /*
     * @desc Parse fn call
     *
     * @grammar
     * Call := f"string{Expression}"
     */
    fn parse(input: Span) -> PResult<Self> {
        todo!();
        // map(pair(wst("f"), parse_fstring::<Expression>), |(_, items)| {
        //     FCall {
        //         value: items
        //             .into_iter()
        //             .map(|item| match item {
        //                 crate::ast::utils::strings::string_parser::FItem::Str(string) => {
        //                     FormatItem::Str(string)
        //                 }
        //                 crate::ast::utils::strings::string_parser::FItem::Expr(expr) => {
        //                     FormatItem::Expr(Expression::Call(Call {
        //                         lib: Some(Core::utils::lexem::STD.to_string().into()),
        //                         fn_var: Box::new(Expression::Atomic(Atomic::Data(Data::Variable(
        //                             Variable {
        //                                 name: Core::utils::lexem::TOSTR.to_string().into(),
        //                                 metadata: Metadata::default(),
        //                                 state: None,
        //                             },
        //                         )))),
        //                         params: vec![expr],
        //                         metadata: Metadata::default(),
        //                         Core: Default::default(),
        //                         is_dynamic_fn: Default::default(),
        //                     }))
        //                 }
        //             })
        //             .collect(),
        //         metadata: Metadata::default(),
        //     }
        // })(input)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        ast::{
            expressions::{
                data::{Data, Number, Primitive, StrSlice, Variable},
                Atomic, Expression,
            },
            statements::{block::Block, return_stat::Return, Statement},
        },
        semantic::Metadata,
        v_num,
    };

    use super::*;

    #[test]
    fn valid_if() {
        let res = IfExpr::parse("if true then 10 else 20".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            IfExpr {
                condition: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(
                    Primitive::Bool(true)
                )))),
                then_branch: ExprBlock::new(vec![
                    (Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))),
                        metadata: Metadata::default()
                    }))
                ]),
                else_branch: ExprBlock::new(vec![
                    (Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 20
                        ))))),
                        metadata: Metadata::default()
                    }))
                ]),
                metadata: Metadata::default(),
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
            case "Hello World" => true,
            case Geo::Point => true,
            case Geo::Point{y} => true,
            // case Point{y} => true,
            // case (y,z) => true,
            else => true
        }"#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_try() {
        let res = TryExpr::parse("try 10 else 20".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(
            TryExpr {
                try_branch: ExprBlock::new(vec![
                    (Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 10
                        ))))),
                        metadata: Metadata::default()
                    }))
                ]),
                else_branch: Some(ExprBlock::new(vec![
                    (Statement::Return(Return::Expr {
                        expr: Box::new(Expression::Atomic(Atomic::Data(Data::Primitive(v_num!(
                            Unresolved, 20
                        ))))),
                        metadata: Metadata::default()
                    }))
                ])),
                pop_last_err: false,
                metadata: Metadata::default(),
            },
            value
        );
    }
}
