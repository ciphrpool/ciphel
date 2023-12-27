use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric1},
    combinator::{map, recognize, value},
    multi::many0_count,
    sequence::pair,
    Parser,
};

use nom_supreme::{tag::complete::tag, ParserExt};

use super::io::{PResult, Span};

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

    use crate::parser::utils::io::{PError, PResult, Span};
    use crate::parser::utils::lexem;

    pub fn cmt<'input, F, O>(inner: F) -> impl FnMut(Span<'input>) -> PResult<O>
    where
        F: Parser<Span<'input>, O, PError<'input>>,
    {
        delimited(ml_comment, inner, ml_comment)
    }

    fn sl_comment(input: Span) -> PResult<()> {
        value(
            (), // Output is thrown away.
            terminated(
                pair(
                    delimited(multispace0, tag(lexem::SL_COMMENT), multispace0),
                    is_not("\n\r\0"),
                ),
                one_of("\n\r\0"),
            ),
        )(input)
    }

    fn ml_comment(input: Span) -> PResult<()> {
        value(
            (),
            many0(alt((
                value(
                    (),
                    tuple((
                        delimited(multispace0, tag(lexem::ML_OP_COMMENT), multispace0),
                        take_until(lexem::ML_CL_COMMENT),
                        delimited(multispace0, tag(lexem::ML_CL_COMMENT), multispace0),
                    )),
                ),
                sl_comment,
            ))),
        )(input)
    }

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
    eater::ws(map(
        eater::ws(recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        ))),
        |_id| _id.to_string(),
    ))
    .context("expected a valid identifier")
    .parse(input)
}

pub fn wst<'input>(lexem: &'static str) -> impl FnMut(Span<'input>) -> PResult<()> {
    move |input: Span| value((), eater::ws(tag(lexem)))(input)
}

/*
 * @desc Parse Escaped String and Character
 *
 * @grammar
 * String := ".*"
 * Character := '.*'
 */
pub mod string_parser {
    use crate::parser::utils::io::{PResult, Span};
    use nom::bytes::complete::take;
    use nom::character::complete::multispace1;
    use nom::combinator::{map_opt, map_res, verify};
    use nom::multi::fold_many0;
    use nom::sequence::preceded;
    use nom::{
        branch::alt,
        bytes::complete::{is_not, take_while_m_n},
        combinator::{map, value},
        sequence::delimited,
        Parser,
    };
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
    use super::{string_parser::parse_string, *};

    #[test]
    fn valid_wst() {
        let res = wst("test")("   test   ".into());
        assert!(res.is_ok());

        let res = pair(wst("test"), wst("test"))("   test   test".into());
        assert!(res.is_ok());

        let res = pair(wst("test"), wst("test"))("   test   test   ".into());
        assert!(res.is_ok());
    }

    #[test]
    fn valid_comments() {
        let res = eater::ws(tag("test"))("   test   ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))(" /* comments */  test   ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))(" // comment \n  test   ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))("  test /* comments */  ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))("  test // comment \n   ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))(" /* comments */  test  /* comments */  ".into());
        assert!(res.is_ok());
        let res = eater::ws(tag("test"))(
            " /* comments */ // comment \n  test  /* comments */ // comment \n  ".into(),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_comments() {
        let res = eater::ws(tag("test"))(" /* comments  test   ".into());
        assert!(res.is_err());
    }

    #[test]
    fn valid_string() {
        let res = parse_string(r#""Hello, World""#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, "Hello, World");

        let res = parse_string(r#""Hello, World\n""#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, "Hello, World\n");

        let res = parse_string(r#""unicode \u{0085}""#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, "unicode \u{0085}");

        let res = parse_string(r#""utf-8 Ɯ 😀""#.into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, "utf-8 Ɯ 😀");
    }

    #[test]
    fn robustness_string() {
        let res = parse_string(r#""Hello," World""#.into());
        assert!(res.is_ok());
        let remainder = res.unwrap().0;
        assert!(!remainder.is_empty());

        let res = parse_string(r#"Hello, World"#.into());
        assert!(res.is_err());
    }
}
