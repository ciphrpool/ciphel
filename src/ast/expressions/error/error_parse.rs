use crate::{
    ast::{
        utils::{
            io::{PResult, Span},
            strings::wst,
        },
        TryParse,
    },
    vm::platform,
};
use nom::combinator::value;

use super::Error;

impl TryParse for Error {
    fn parse(input: Span) -> PResult<Self> {
        value(Error(), wst(platform::utils::lexem::ERROR))(input)
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_error() {
        let res = Error::parse("error".into());
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Error(), value);
    }
}
