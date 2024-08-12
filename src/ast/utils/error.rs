use nom::{error::ParseError, IResult};

use super::io::{PResult, Span};

#[derive(Debug)]
struct CutError<E> {
    error: E,
    should_cut: bool,
}

// Implement ParseError for our custom error type
impl<E: ParseError<I>, I> ParseError<I> for CutError<E> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        CutError {
            error: E::from_error_kind(input, kind),
            should_cut: false,
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        CutError {
            error: E::append(input, kind, other.error),
            should_cut: other.should_cut,
        }
    }
}

// Our custom cut_after function
fn cut_after<I, O, E, F>(mut parser: F) -> impl FnMut(I) -> IResult<I, O, CutError<E>>
where
    I: Clone,
    E: ParseError<I>,
    F: FnMut(I) -> IResult<I, O, E>,
{
    move |input: I| match parser(input) {
        Ok((rem, output)) => Ok((rem, output)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(CutError {
            error: e,
            should_cut: true,
        })),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(CutError {
            error: e,
            should_cut: true,
        })),
        Err(nom::Err::Incomplete(n)) => Err(nom::Err::Incomplete(n)),
    }
}

pub fn squash<'input, O, F>(
    mut parser: F,
    context: &'static str,
) -> impl FnMut(Span<'input>) -> PResult<O>
where
    F: FnMut(Span<'input>) -> PResult<O>,
{
    move |input: Span<'input>| match parser(input) {
        Ok(succ) => Ok(succ),
        Err(nom::Err::Error(e)) => {
            drop(e);
            Err(nom::Err::Error(
                nom_supreme::error::GenericErrorTree::Stack {
                    base: Box::new(nom_supreme::error::GenericErrorTree::Base {
                        location: input,
                        kind: nom_supreme::error::BaseErrorKind::Expected(
                            nom_supreme::error::Expectation::Something,
                        ),
                    }),
                    contexts: vec![(input, nom_supreme::error::StackContext::Context(context))],
                },
            ))
        }
        Err(err) => Err(err),
    }
}
