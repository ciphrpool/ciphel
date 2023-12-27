use nom_supreme::error::ErrorTree;

pub type Span<'input> = nom_locate::LocatedSpan<&'input str>;
pub type PResult<'input, Output> = nom::IResult<Span<'input>, Output, ErrorTree<Span<'input>>>;
pub type PError<'input> = ErrorTree<Span<'input>>;
