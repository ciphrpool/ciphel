use nom::{
    combinator::map,
    sequence::{delimited, preceded},
};

use crate::parser::{
    ast::TryParse,
    utils::{
        io::{PResult, Span},
        lexem,
        numbers::parse_number,
        strings::wst,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Error(usize);

impl TryParse for Error {
    fn parse(input: Span) -> PResult<Self> {
        map(
            preceded(
                wst(lexem::platform::ERROR),
                delimited(wst(lexem::PAR_O), parse_number, wst(lexem::PAR_C)),
            ),
            |value| Error(value.unsigned_abs() as usize),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_error() {
        let res = Error::parse("error(10)".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Error(10), value);
    }
}
