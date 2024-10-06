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
    utils::{
        error::{generate_error_report, squash},
        strings::eater,
    },
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
pub struct WithLine<T> {
    pub inner: T,
    pub line: usize,
}

impl<T: Resolve> Resolve for WithLine<T> {
    type Output = T::Output;
    type Context = T::Context;
    type Extra = T::Extra;

    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.inner.resolve::<G>(scope, context, extra)
    }
}

impl<T: GenerateCode> GenerateCode for WithLine<T> {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &mut CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        self.inner.gencode(scope, instructions)
    }
}

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

pub fn parse_statements(
    input: Span,
    line_offset: usize,
) -> Result<Vec<WithLine<Statement>>, CompilationError> {
    let mut parser = many1(|input| {
        let (input, statement) = Statement::parse(input)?;
        let line = input.location_line();
        Ok((
            input,
            WithLine {
                inner: statement,
                line: line as usize + line_offset,
            },
        ))
    });

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
            let error_report = generate_error_report(&input, &e, line_offset);
            Err(CompilationError::ParsingError(error_report))
        }
    }
}

impl TryParse for Statement {
    fn parse(input: Span) -> PResult<Self> {
        eater::ws(squash(
            alt((
                map(Return::parse, Statement::Return),
                map(block::Block::parse, Statement::Scope),
                map(declaration::Declaration::parse, Statement::Declaration),
                map(flows::Flow::parse, Statement::Flow),
                map(definition::Definition::parse, Statement::Definition),
                map(loops::Loop::parse, Statement::Loops),
                map(assignation::Assignation::parse, Statement::Assignation),
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
