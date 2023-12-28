use super::io::{PResult, Span};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, many1},
    sequence::{preceded, terminated, tuple},
    IResult, Parser,
};
use nom_supreme::error::ErrorTree;
use num_traits::Num;

fn hexadecimal(input: Span) -> PResult<Span> {
    preceded(
        alt((tag("0x"), tag("0X"))),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )(input)
}
fn octal(input: Span) -> PResult<Span> {
    preceded(
        alt((tag("0o"), tag("0O"))),
        recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
    )(input)
}
fn binary(input: Span) -> PResult<Span> {
    preceded(
        alt((tag("0b"), tag("0B"))),
        recognize(many1(terminated(one_of("01"), many0(char('_'))))),
    )(input)
}
fn decimal(input: Span) -> PResult<Span> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

fn float_decimal(input: Span) -> IResult<Span, Span> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

fn float(input: Span) -> IResult<Span, Span> {
    alt((
        // Case one: .42
        recognize(tuple((
            opt(one_of("+-")),
            char('.'),
            float_decimal,
            opt(tuple((one_of("eE"), opt(one_of("+-")), float_decimal))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            opt(one_of("+-")),
            float_decimal,
            opt(preceded(char('.'), float_decimal)),
            one_of("eE"),
            opt(one_of("+-")),
            float_decimal,
        ))), // Case three: 42. and 42.42
        recognize(tuple((
            opt(one_of("+-")),
            float_decimal,
            char('.'),
            opt(float_decimal),
        ))),
    ))(input)
}

pub type Number = i64;

pub fn parse_number(input: Span) -> PResult<Number> {
    alt((
        map_res(octal, |value| {
            Number::from_str_radix(&value.replace("_", ""), 8)
        }),
        map_res(hexadecimal, |value| {
            Number::from_str_radix(&value.replace("_", ""), 16)
        }),
        map_res(binary, |value| {
            Number::from_str_radix(&value.replace("_", ""), 2)
        }),
        map_res(decimal, |value| {
            Number::from_str_radix(&value.replace("_", ""), 10)
        }),
    ))(input)
}

fn _parse_float(input: Span) -> IResult<Span, f64> {
    map_res(float, |value| {
        f64::from_str_radix(&value.replace("_", ""), 10)
    })(input)
}

pub fn parse_float(input: Span) -> PResult<f64> {
    let res = _parse_float(input);
    match res {
        Ok((remainder, value)) => Ok((remainder, value)),
        Err(e) => match e {
            nom::Err::Incomplete(e) => Err(nom::Err::Incomplete(e)),
            nom::Err::Error(e) => Err(nom::Err::Error(ErrorTree::Base {
                location: e.input,
                kind: nom_supreme::error::BaseErrorKind::Kind(e.code),
            })),
            nom::Err::Failure(e) => Err(nom::Err::Failure(ErrorTree::Base {
                location: e.input,
                kind: nom_supreme::error::BaseErrorKind::Kind(e.code),
            })),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer() {
        let res = parse_number("0".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0u32.into());

        let res = parse_number("0000_000_0".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0u32.into());

        let res = parse_number("12345".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 12345u32.into());

        let res = parse_number("0b101010".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 42u32.into());

        let res = parse_number("0x2a".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 42u32.into());

        let res = parse_number("0o52".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 42u32.into());
    }

    #[test]
    fn float() {
        let res = parse_float(".42".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0.42);
        let res = parse_float("41.43".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 41.43);
        let res = parse_float("42.".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 42.);
        let res = parse_float("6.0".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 6.);

        let res = parse_float(".42e10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0.42e10);
        let res = parse_float(".42e+10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0.42e10);
        let res = parse_float(".42e-10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 0.42e-10);

        let res = parse_float("41.43e10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 41.43e10);
        let res = parse_float("41.43e+10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 41.43e10);
        let res = parse_float("41.43e-10".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(value, 41.43e-10);
    }
}
