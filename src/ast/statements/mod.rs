use nom::{
    branch::alt,
    combinator::{eof, map},
    multi::{many0, many1},
    sequence::terminated,
    Finish, Parser,
};
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation, GenericErrorTree, StackContext};

use self::return_stat::Return;
use crate::{
    semantic::scope::scope::ScopeManager, vm::vm::CodeGenerationContext, CompilationError,
};

use super::{
    utils::{
        error::{generate_error_report, squash},
        strings::eater,
    },
    TryParse,
};
use crate::vm::vm::GenerateCode;
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{scope::static_types::StaticType, EType, Resolve, SemanticError, TypeOf},
    vm::{
        casm::{
            branch::{Call, Goto, Label},
            Casm, CasmProgram,
        },
        vm::CodeGenerationError,
    },
};

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
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.inner
            .resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

impl<T: GenerateCode> GenerateCode for WithLine<T> {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        self.inner
            .gencode(scope_manager, scope_id, instructions, context)
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
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Statement::Scope(value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut ())?;
                Ok(())
            }
            Statement::Flow(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Statement::Assignation(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Statement::Declaration(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Statement::Definition(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
            Statement::Loops(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Statement::Return(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
        }
    }
}

impl TypeOf for Statement {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope_manager, scope_id),
            Statement::Flow(value) => value.type_of(&scope_manager, scope_id),
            Statement::Assignation(value) => value.type_of(&scope_manager, scope_id),
            Statement::Declaration(value) => value.type_of(&scope_manager, scope_id),
            Statement::Definition(_value) => Ok(EType::Static(StaticType::Unit)),
            Statement::Loops(value) => value.type_of(&scope_manager, scope_id),
            Statement::Return(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl GenerateCode for Statement {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Statement::Scope(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)?;
                Ok(())
            }
            Statement::Flow(value) => value.gencode(scope_manager, scope_id, instructions, context),
            Statement::Assignation(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Statement::Declaration(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Statement::Definition(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Statement::Loops(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Statement::Return(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
        }
    }
}
