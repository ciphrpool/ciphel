use crate::{
    ast::{
        utils::{
            io::{PResult, Span},
            lexem,
            numbers::parse_number,
            strings::{eater, wst},
        },
        TryParse,
    },
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError},
};
use nom::{
    combinator::{map, value},
    sequence::{delimited, preceded},
};

use super::Error;

impl TryParse for Error {
    fn parse(input: Span) -> PResult<Self> {
        value(Error(), wst(lexem::platform::ERROR))(input)
    }
}
#[cfg(test)]
mod tests {
    use crate::ast::expressions::Expression;

    use super::*;

    #[test]
    fn valid_error() {
        let res = Error::parse("error".into());
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Error(), value);
    }
}
