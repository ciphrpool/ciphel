use std::sync::Arc;

use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric1, anychar},
    combinator::{map_res, not, peek, recognize, value, verify},
    multi::many0_count,
    sequence::{pair, terminated},
    Parser,
};

use nom_supreme::{tag::complete::tag, ParserExt};

use super::{
    io::{PResult, Span},
    lexem,
};

/*
 * @desc Eat whitespace and comments
 *
 * @grammar
 * Comments := /* .* */ | // .* \n
 */
pub mod eater {
    use nom::{
        branch::alt,
        bytes::complete::{is_not, take_until},
        character::complete::{multispace0, one_of},
        combinator::value,
        multi::many0,
        sequence::{delimited, pair, terminated, tuple},
        Parser,
    };
    use nom_supreme::tag::complete::tag;

    use crate::ast::utils::{
        io::{PError, PResult, Span},
        lexem,
    };
    #[inline]
    pub fn cmt<'input, F, O>(inner: F) -> impl FnMut(Span<'input>) -> PResult<O>
    where
        F: Parser<Span<'input>, O, PError<'input>>,
    {
        delimited(ml_comment, inner, ml_comment)
    }
    #[inline]
    fn sl_comment(input: Span) -> PResult<()> {
        value(
            (), // Output is thrown away.
            terminated(
                pair(tag(lexem::SL_COMMENT), is_not("\n\r\0")),
                one_of("\n\r\0"),
            ),
        )(input)
    }
    #[inline]
    pub fn ml_comment(input: Span) -> PResult<()> {
        value(
            (),
            many0(delimited(
                multispace0,
                alt((
                    value(
                        (),
                        tuple((
                            tag(lexem::ML_OP_COMMENT),
                            take_until(lexem::ML_CL_COMMENT),
                            tag(lexem::ML_CL_COMMENT),
                        )),
                    ),
                    sl_comment,
                )),
                multispace0,
            )),
        )(input)
    }

    #[inline]
    pub fn ws<'input, F, O>(inner: F) -> impl FnMut(Span<'input>) -> PResult<O>
    where
        F: Parser<Span<'input>, O, PError<'input>>,
    {
        delimited(
            delimited(multispace0, ml_comment, multispace0),
            inner,
            delimited(multispace0, ml_comment, multispace0),
        )
    }
}

pub type ID = String;

/*
 * @desc Parse Identifier
 *
 * @grammar
 * [a-zA-Z_][a-zA-Z_0-9]*
 */
pub fn parse_id(input: Span) -> PResult<ID> {
    eater::ws(map_res(
        eater::ws(recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        ))),
        |_id| {
            let id = _id.to_string();
            let is_in_lexem = match id.as_str() {
                lexem::ENUM => true,
                lexem::STRUCT => true,
                lexem::UNION => true,
                lexem::AS => true,
                lexem::ELSE => true,
                lexem::TRY => true,
                // platform::utils::lexem::ERROR => true,
                // platform::utils::lexem::OK => true,
                lexem::IF => true,
                lexem::THEN => true,
                lexem::MATCH => true,
                lexem::CASE => true,
                lexem::RETURN => true,
                lexem::LET => true,
                lexem::WHILE => true,
                lexem::FOR => true,
                lexem::LOOP => true,
                lexem::BREAK => true,
                lexem::CONTINUE => true,
                lexem::MOVE => true,
                lexem::REC => true,
                lexem::DYN => true,
                lexem::U8 => true,
                lexem::U16 => true,
                lexem::U32 => true,
                lexem::U64 => true,
                lexem::U128 => true,
                lexem::I8 => true,
                lexem::I16 => true,
                lexem::I32 => true,
                lexem::I64 => true,
                lexem::I128 => true,
                lexem::FLOAT => true,
                lexem::CHAR => true,
                lexem::STRING => true,
                lexem::STR => true,
                lexem::BOOL => true,
                lexem::UNIT => true,
                lexem::ANY => true,
                lexem::ERR => true,
                lexem::UUNIT => true,
                lexem::UVEC => true,
                lexem::UMAP => true,
                lexem::RANGE_I => true,
                lexem::RANGE_E => true,
                lexem::GENERATOR => true,
                lexem::FN => true,
                lexem::TRUE => true,
                lexem::FALSE => true,
                _ => false,
            };
            if is_in_lexem {
                Err(nom::Err::Error(nom::error::Error::new(
                    id.as_str().to_string(),
                    nom::error::ErrorKind::AlphaNumeric,
                )))
            } else {
                Ok(id)
            }
        },
    ))
    .context("expected a valid identifier")
    .parse(input)
}

pub fn wst<'input>(lexem: &'static str) -> impl FnMut(Span<'input>) -> PResult<()> {
    move |input: Span| value((), eater::ws(tag(lexem)))(input)
}
pub fn keyword<'input>(kw: &'static str) -> impl FnMut(Span<'input>) -> PResult<()> {
    move |input: Span| {
        value(
            (),
            terminated(
                tag(kw),
                peek(not(verify(anychar, |c: &char| {
                    c.is_ascii_alphanumeric() || *c == '_'
                }))),
            ),
        )(input)
    }
}
pub fn wst_closed<'input>(lexem: &'static str) -> impl FnMut(Span<'input>) -> PResult<()> {
    move |input: Span| value((), eater::ws(keyword(lexem)))(input)
}
/*
 * @desc Parse Escaped String and Character
 *
 * @grammar
 * String := ".*"
 * Character := '.*'
 */
pub mod string_parser {
    use crate::ast::utils::io::{PResult, Span};
    use crate::ast::utils::lexem;
    use crate::ast::TryParse;
    use nom::bytes::complete::take;
    use nom::character::complete::{multispace0, multispace1};
    use nom::combinator::{map_opt, map_res, verify};
    use nom::multi::fold_many0;
    use nom::sequence::{preceded, terminated};
    use nom::{
        branch::alt,
        bytes::complete::{is_not, take_while_m_n},
        combinator::{map, value},
        sequence::delimited,
        Parser,
    };
    use nom_supreme::tag::complete::tag;
    use nom_supreme::ParserExt;

    use super::eater::ml_comment;
    fn unicode(input: Span) -> PResult<char> {
        let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
        let parse_delimited_hex = preceded(
            nom::character::complete::char('u'),
            delimited(
                nom::character::complete::char('{'),
                parse_hex,
                nom::character::complete::char('}'),
            ),
        );
        let parse_u32 = map_res(parse_delimited_hex, move |hex: Span| {
            u32::from_str_radix(hex.to_string().as_str(), 16)
        });
        map_opt(parse_u32, std::char::from_u32).parse(input)
    }

    fn escaped_char<'input>(input: Span<'input>) -> PResult<char> {
        preceded(
            nom::character::complete::char('\\'),
            alt((
                unicode,
                value('\n', nom::character::complete::char('n')),
                value('\r', nom::character::complete::char('r')),
                value('\t', nom::character::complete::char('t')),
                value('\u{08}', nom::character::complete::char('b')),
                value('\u{0C}', nom::character::complete::char('f')),
                value('\\', nom::character::complete::char('\\')),
                value('/', nom::character::complete::char('/')),
                value('"', nom::character::complete::char('"')),
            )),
        )
        .parse(input)
    }

    fn escaped_whitespace(input: Span) -> PResult<Span> {
        preceded(nom::character::complete::char('\\'), multispace1).parse(input)
    }

    fn parse_literal(input: Span) -> PResult<Span> {
        let not_quote_slash = is_not("\"\\");
        verify(not_quote_slash, |s: &Span| !s.is_empty()).parse(input)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum StringFragment<'input> {
        Literal(Span<'input>),
        EscapedChar(char),
        EscapedWS,
    }

    fn fragment(input: Span) -> PResult<StringFragment> {
        alt((
            map(parse_literal, StringFragment::Literal),
            map(escaped_char, StringFragment::EscapedChar),
            value(StringFragment::EscapedWS, escaped_whitespace),
        ))
        .parse(input)
    }

    pub fn parse_string(input: Span) -> PResult<String> {
        let build_string = fold_many0(fragment, String::new, |mut string, frag| {
            match frag {
                StringFragment::Literal(s) => string.push_str(s.to_string().as_str()),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        });

        delimited(
            nom::character::complete::char('"'),
            build_string,
            nom::character::complete::char('"'),
        )
        .context("Invalid string")
        .parse(input)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FormattedStringFragment<'input, E: TryParse + Clone> {
        Literal(Span<'input>),
        EscapedChar(char),
        EscapedWS,
        Expr(E),
        BraO,
        BraC,
    }
    fn parse_formatted_literal(input: Span) -> PResult<Span> {
        let not_quote_slash = is_not("\"\\{}");
        verify(not_quote_slash, |s: &Span| !s.is_empty()).parse(input)
    }

    fn formatted_fragment<'input, E: TryParse + Clone>(
        input: Span<'input>,
    ) -> PResult<FormattedStringFragment<'input, E>> {
        alt((
            map(tag("{{"), |_| FormattedStringFragment::BraO),
            map(tag("}}"), |_| FormattedStringFragment::BraC),
            map(
                delimited(
                    terminated(
                        tag(lexem::BRA_O),
                        delimited(multispace0, ml_comment, multispace0),
                    ),
                    E::parse,
                    preceded(
                        delimited(multispace0, ml_comment, multispace0),
                        tag(lexem::BRA_C),
                    ),
                ),
                FormattedStringFragment::Expr,
            ),
            map(parse_formatted_literal, FormattedStringFragment::Literal),
            map(escaped_char, FormattedStringFragment::EscapedChar),
            value(FormattedStringFragment::EscapedWS, escaped_whitespace),
        ))
        .parse(input)
    }
    #[derive(Debug, Clone, PartialEq)]
    pub enum FItem<E: TryParse + Clone> {
        Str(String),
        Expr(E),
    }
    pub fn parse_fstring<E: TryParse + Clone>(input: Span) -> PResult<Vec<FItem<E>>> {
        let build_string = fold_many0(formatted_fragment::<E>, Vec::new, |mut items, frag| {
            match frag {
                FormattedStringFragment::Literal(s) => match items.last_mut() {
                    Some(FItem::Str(string)) => string.push_str(s.to_string().as_str()),
                    Some(FItem::Expr(_)) => items.push(FItem::Str(s.to_string())),
                    None => items.push(FItem::Str(s.to_string())),
                },
                FormattedStringFragment::EscapedChar(c) => match items.last_mut() {
                    Some(FItem::Str(string)) => string.push(c),
                    Some(FItem::Expr(_)) => items.push(FItem::Str(c.to_string())),
                    None => items.push(FItem::Str(c.to_string())),
                },
                FormattedStringFragment::EscapedWS => {}
                FormattedStringFragment::Expr(e) => match items.last_mut() {
                    Some(FItem::Str(_)) => items.push(FItem::Expr(e)),
                    Some(FItem::Expr(_)) => items.push(FItem::Expr(e)),
                    None => items.push(FItem::Str('{'.to_string())),
                },
                FormattedStringFragment::BraO => match items.last_mut() {
                    Some(FItem::Str(string)) => string.push('{'),
                    Some(FItem::Expr(_)) => items.push(FItem::Str('{'.to_string())),
                    None => items.push(FItem::Str('{'.to_string())),
                },
                FormattedStringFragment::BraC => match items.last_mut() {
                    Some(FItem::Str(string)) => string.push('}'),
                    Some(FItem::Expr(_)) => items.push(FItem::Str('}'.to_string())),
                    None => items.push(FItem::Str('}'.to_string())),
                },
            }
            items
        });

        delimited(
            nom::character::complete::char('"'),
            build_string,
            nom::character::complete::char('"'),
        )(input)
    }

    pub fn parse_char(input: Span) -> PResult<char> {
        delimited(
            nom::character::complete::char('\''),
            alt((
                escaped_char,
                map_res(take(1usize), |out: Span| str::parse::<char>(&out)),
            )),
            nom::character::complete::char('\''),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use tests::string_parser::FItem;

    use crate::ast::expressions::Expression;

    use self::string_parser::parse_fstring;

    use super::{string_parser::parse_string, *};

    #[test]
    fn valid_wst() {
        let res = wst("test")("   test   ".into());
        assert!(res.is_ok(), "{:?}", res);

        let res = pair(wst("test"), wst("test"))("   test   test".into());
        assert!(res.is_ok(), "{:?}", res);

        let res = pair(wst("test"), wst("test"))("   test   test   ".into());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_comments() {
        let res = eater::ws(tag("test"))("   test   ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))(" /* comments */  test   ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))(" // comment \n  test   ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))("  test /* comments */  ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))("  test // comment \n   ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))(" /* comments */  test  /* comments */  ".into());
        assert!(res.is_ok(), "{:?}", res);
        let res = eater::ws(tag("test"))(
            " /* comments */ // comment \n  test  /* comments */ // comment \n  ".into(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_comments() {
        let res = eater::ws(tag("test"))(" /* comments  test   ".into());
        assert!(res.is_err());
    }

    #[test]
    fn valid_string() {
        let res = parse_string(r#""Hello, World""#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(value, "Hello, World");

        let res = parse_string(r#""Hello, World\n""#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(value, "Hello, World\n");

        let res = parse_string(r#""unicode \u{0085}""#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(value, "unicode \u{0085}");

        let res = parse_string(r#""utf-8 Ɯ 😀""#.into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(value, "utf-8 Ɯ 😀");
    }

    #[test]
    fn robustness_string() {
        let res = parse_string(r#""Hello," World""#.into());
        assert!(res.is_ok(), "{:?}", res);
        let remainder = res.unwrap().0;
        assert!(!remainder.is_empty());

        let res = parse_string(r#"Hello, World"#.into());
        assert!(res.is_err());
    }

    #[test]
    fn valid_formatted_string() {
        let res = parse_fstring::<Expression>(r#""Hello,{2} World""#.into())
            .expect("Parsing should have succeeded")
            .1;
        assert_eq!(res.len(), 3);
        let res = parse_fstring::<Expression>(r#""Hello,{ x  } World""#.into())
            .expect("Parsing should have succeeded")
            .1;
        assert_eq!(res.len(), 3);
        let res = parse_fstring::<Expression>(r#""Hello,{{2}} World""#.into())
            .expect("Parsing should have succeeded")
            .1;
        assert_eq!(res, vec![FItem::Str("Hello,{2} World".into())]);

        let res = parse_fstring::<Expression>(r#""Hello,{{  World""#.into())
            .expect("Parsing should have succeeded")
            .1;
        assert_eq!(res, vec![FItem::Str("Hello,{  World".into())]);

        let res = parse_fstring::<Expression>(r#""Hello, }}  World""#.into())
            .expect("Parsing should have succeeded")
            .1;
        assert_eq!(res, vec![FItem::Str("Hello, }  World".into())]);

        let _ = parse_fstring::<Expression>(r#""Hello,{ World""#.into())
            .expect_err("Parsing should have succeeded");
        let _ = parse_fstring::<Expression>(r#""Hello,} World""#.into())
            .expect_err("Parsing should have succeeded");
    }
}
