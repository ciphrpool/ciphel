use nom::{error::ParseError, IResult};
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation, GenericErrorTree, StackContext};

use super::io::{PResult, Span};

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

pub fn generate_error_report(input: &Span, error: &ErrorTree<Span>, line_offset: usize) -> String {
    let mut report = String::new();

    fn traverse_error<'a>(
        input: &Span,
        error: &'a ErrorTree<Span>,
        report: &mut String,
        depth: usize,
        line_offset: usize,
    ) {
        match error {
            GenericErrorTree::Base { location, kind } => {
                add_error_info(input, location, kind, report, depth, line_offset);
            }
            GenericErrorTree::Stack { base, contexts } => {
                for (ctx_location, ctx) in contexts.iter().rev() {
                    add_context_info(input, ctx_location, ctx, report, depth, line_offset);
                }
                traverse_error(input, base, report, depth, line_offset);
            }
            GenericErrorTree::Alt(errors) => {
                for err in errors {
                    traverse_error(input, err, report, depth, line_offset);
                }
            }
        }
    }

    traverse_error(input, error, &mut report, 0, line_offset);
    report
}

pub fn add_error_info(
    input: &Span,
    location: &Span,
    kind: &BaseErrorKind<&str, Box<dyn std::error::Error + Send + Sync>>,
    report: &mut String,
    depth: usize,
    line_offset: usize,
) {
    let line = location.location_line() + line_offset as u32;
    let column = location.get_utf8_column();
    let snippet = get_error_snippet(input, location);
    let text = format_error_kind(kind);
    let indent = "\t".repeat(depth);
    if text.is_empty() {
        return;
    }

    report.push_str(&format!(
        "{}Error at line {}, column {}:\n",
        indent, line, column
    ));
    report.push_str(&format!("{}  {}\n", indent, snippet));
    report.push_str(&format!("{}  {}\n\n", indent, text));
}

pub fn add_context_info(
    input: &Span,
    location: &Span,
    context: &StackContext<&str>,
    report: &mut String,
    depth: usize,
    line_offset: usize,
) {
    let line = location.location_line() + line_offset as u32;
    let column = location.get_utf8_column();
    let snippet = get_error_snippet(input, location);

    let indent = "\t".repeat(depth);
    let text = match context {
        StackContext::Kind(nom::error::ErrorKind::Many1) => return,
        StackContext::Kind(_) => context.to_string(),
        StackContext::Context(text) => text.to_string(),
    };
    report.push_str(&format!(
        "{}Error at line {}, column {}:\n",
        indent, line, column
    ));
    report.push_str(&format!("{}  {}\n", indent, snippet));
    report.push_str(&format!("{}  {}\n\n", indent, text));
}

pub fn format_error_kind(
    kind: &BaseErrorKind<&str, Box<dyn std::error::Error + Send + Sync>>,
) -> String {
    match kind {
        BaseErrorKind::Expected(Expectation::Something) => "".to_string(),
        BaseErrorKind::Expected(expected) => format!("Expected {}", expected),
        BaseErrorKind::External(err) => format!("{}", err),
        _ => format!("{:?}", kind),
    }
}

pub fn get_error_snippet(input: &Span, location: &Span) -> String {
    let start = location.location_offset();
    let end = (start + 20).min(input.len());
    let snippet = &input[start..end];
    format!("{:?}", snippet)
}
