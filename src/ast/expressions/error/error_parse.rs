use crate::ast::{
    utils::{
        io::{PResult, Span},
        lexem,
        strings::wst,
    },
    TryParse,
};
use nom::combinator::value;

use super::Error;

impl TryParse for Error {
    fn parse(input: Span) -> PResult<Self> {
        value(Error(), wst(lexem::platform::ERROR))(input)
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_error() {
        let res = Error::parse("error".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Error(), value);
    }
}
