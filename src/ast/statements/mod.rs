use nom::{
    branch::alt,
    combinator::{eof, map},
    multi::{many0, many1},
    sequence::terminated,
    Finish, Parser,
};
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation, GenericErrorTree, StackContext};

use self::return_stat::Return;
use crate::{semantic::scope::scope::Scope, CompilationError};

use super::{
    utils::{error::squash, strings::eater},
    TryParse,
};
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{scope::static_types::StaticType, EType, Either, Resolve, SemanticError, TypeOf},
    vm::{
        casm::{
            branch::{Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::CodeGenerationError,
    },
};
use crate::{semantic::scope::BuildStaticType, vm::vm::GenerateCode};

pub mod assignation;
pub mod block;
pub mod declaration;
pub mod definition;
pub mod flows;
pub mod loops;
pub mod return_stat;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Scope(block::Block),
    Flow(flows::Flow),
    Assignation(assignation::Assignation),
    Declaration(declaration::Declaration),
    Definition(definition::Definition),
    Loops(loops::Loop),
    Return(Return),
}

pub fn parse_statements(input: Span) -> Result<Vec<Statement>, CompilationError> {
    let mut parser = many1(Statement::parse);

    match parser(input).finish() {
        Ok((remaining, statements)) => {
            if !remaining.fragment().is_empty() {
                let error_report = format!(
                    "Parsing completed, but input remains: {:?}",
                    remaining.fragment()
                );
                Err(CompilationError::ParsingError(error_report))
            } else {
                Ok(statements)
            }
        }
        Err(e) => {
            let error_report = generate_error_report(&input, &e);
            Err(CompilationError::ParsingError(error_report))
        }
    }
}
fn generate_error_report(input: &Span, error: &ErrorTree<Span>) -> String {
    let mut report = String::new();

    fn traverse_error<'a>(
        input: &Span,
        error: &'a ErrorTree<Span>,
        report: &mut String,
        depth: usize,
    ) {
        match error {
            GenericErrorTree::Base { location, kind } => {
                add_error_info(input, location, kind, report, depth);
            }
            GenericErrorTree::Stack { base, contexts } => {
                for (ctx_location, ctx) in contexts.iter().rev() {
                    add_context_info(input, ctx_location, ctx, report, depth);
                }
                traverse_error(input, base, report, depth);
            }
            GenericErrorTree::Alt(errors) => {
                for err in errors {
                    traverse_error(input, err, report, depth);
                }
            }
        }
    }

    traverse_error(input, error, &mut report, 0);
    report
}

fn add_error_info(
    input: &Span,
    location: &Span,
    kind: &BaseErrorKind<&str, Box<dyn std::error::Error + Send + Sync>>,
    report: &mut String,
    depth: usize,
) {
    let line = location.location_line();
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

fn add_context_info(
    input: &Span,
    location: &Span,
    context: &StackContext<&str>,
    report: &mut String,
    depth: usize,
) {
    let line = location.location_line();
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

fn format_error_kind(
    kind: &BaseErrorKind<&str, Box<dyn std::error::Error + Send + Sync>>,
) -> String {
    match kind {
        BaseErrorKind::Expected(Expectation::Something) => "".to_string(),
        BaseErrorKind::Expected(expected) => format!("Expected {}", expected),
        BaseErrorKind::External(err) => format!("{}", err),
        _ => format!("{:?}", kind),
    }
}

fn get_error_snippet(input: &Span, location: &Span) -> String {
    let start = location.location_offset();
    let end = (start + 20).min(input.len());
    let snippet = &input[start..end];
    format!("{:?}", snippet)
}

impl TryParse for Statement {
    fn parse(input: Span) -> PResult<Self> {
        eater::ws(squash(
            alt((
                map(Return::parse, Statement::Return),
                map(block::Block::parse, Statement::Scope),
                map(assignation::Assignation::parse, Statement::Assignation),
                map(flows::Flow::parse, Statement::Flow),
                map(declaration::Declaration::parse, Statement::Declaration),
                map(definition::Definition::parse, Statement::Definition),
                map(loops::Loop::parse, Statement::Loops),
            )),
            "expected a valid statements",
        ))(input)
    }
}
impl Resolve for Statement {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Statement::Scope(value) => {
                let _ = value.resolve::<G>(scope, context, &mut Vec::default())?;
                Ok(())
            }
            Statement::Flow(value) => value.resolve::<G>(scope, context, extra),
            Statement::Assignation(value) => value.resolve::<G>(scope, context, extra),
            Statement::Declaration(value) => value.resolve::<G>(scope, context, extra),
            Statement::Definition(value) => value.resolve::<G>(scope, context, extra),
            Statement::Loops(value) => value.resolve::<G>(scope, context, extra),
            Statement::Return(value) => value.resolve::<G>(scope, context, extra),
        }
    }
}

impl TypeOf for Statement {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope),
            Statement::Flow(value) => value.type_of(&scope),
            Statement::Assignation(value) => value.type_of(&scope),
            Statement::Declaration(value) => value.type_of(&scope),
            Statement::Definition(_value) => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Statement::Loops(value) => value.type_of(&scope),
            Statement::Return(value) => value.type_of(&scope),
        }
    }
}

impl GenerateCode for Statement {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Statement::Scope(value) => {
                let scope_label = Label::gen();
                let end_scope_label = Label::gen();

                instructions.push(Casm::Goto(Goto {
                    label: Some(end_scope_label),
                }));
                instructions.push_label_id(scope_label, "block".to_string().into());

                let _ = value.gencode(scope, instructions)?;

                instructions.push_label_id(end_scope_label, "end_scope".to_string().into());
                instructions.push(Casm::Call(Call::From {
                    label: scope_label,
                    param_size: 0,
                }));
                instructions.push(Casm::Pop(9)); /* Pop the unused return size and return flag */
                Ok(())
            }
            Statement::Flow(value) => value.gencode(scope, instructions),
            Statement::Assignation(value) => value.gencode(scope, instructions),
            Statement::Declaration(value) => value.gencode(scope, instructions),
            Statement::Definition(value) => value.gencode(scope, instructions),
            Statement::Loops(value) => value.gencode(scope, instructions),
            Statement::Return(value) => value.gencode(scope, instructions),
        }
    }
}
