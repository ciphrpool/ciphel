use nom::{branch::alt, combinator::map, multi::many0, Finish, Parser};

use self::return_stat::Return;
use crate::{
    semantic::{Desugar, Metadata},
    vm::{
        external::{ExternProcessIdentifier, ExternThreadIdentifier},
        GenerateCode,
    },
    CompilationError,
};

use super::{
    expressions::Expression,
    utils::{
        error::{generate_error_report, squash},
        strings::eater::{self, ws},
    },
    TryParse,
};
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{scope::static_types::StaticType, EType, Resolve, SemanticError, TypeOf},
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

    fn resolve<E: crate::vm::external::Engine>(
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
            .resolve::<E>(scope_manager, scope_id, context, extra)
    }
}

impl<T: Desugar<()>> Desugar<()> for WithLine<T> {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<()>, SemanticError> {
        self.inner.desugar::<E>(scope_manager, scope_id)
    }
}

impl<T: GenerateCode> GenerateCode for WithLine<T> {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        self.inner
            .gencode::<E>(scope_manager, scope_id, instructions, context)
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

pub fn parse_statements<PID: ExternProcessIdentifier, TID: ExternThreadIdentifier<PID>>(
    input: Span,
    line_offset: usize,
) -> Result<Vec<WithLine<Statement>>, CompilationError<PID, TID>> {
    let mut parser = ws(many0(|input| {
        let (input, statement) = Statement::parse(input)?;
        let line = input.location_line();
        Ok((
            input,
            WithLine {
                inner: statement,
                line: line as usize + line_offset,
            },
        ))
    }));

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
                map(Expression::parse, |expr| {
                    Statement::Return(Return::Inline {
                        expr: Box::new(expr),
                        metadata: Metadata::default(),
                    })
                }),
            )),
            "expected a valid statements",
        ))(input)
    }
}
impl Resolve for Statement {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        if let Some(output) = self.desugar::<E>(scope_manager, scope_id)? {
            *self = output;
        }

        match self {
            Statement::Scope(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, &mut ())
            }
            Statement::Flow(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Statement::Assignation(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Statement::Declaration(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Statement::Definition(value) => {
                value.resolve::<E>(scope_manager, scope_id, context, extra)
            }
            Statement::Loops(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Statement::Return(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
        }
    }
}

impl Desugar<Statement> for Statement {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        let output = match self {
            Statement::Scope(block) => block.desugar::<E>(scope_manager, scope_id)?,
            Statement::Flow(flow) => flow.desugar::<E>(scope_manager, scope_id)?,
            Statement::Assignation(assignation) => {
                assignation.desugar::<E>(scope_manager, scope_id)?
            }
            Statement::Declaration(declaration) => {
                declaration.desugar::<E>(scope_manager, scope_id)?
            }
            Statement::Definition(definition) => {
                definition.desugar::<E>(scope_manager, scope_id)?
            }
            Statement::Loops(loops) => loops.desugar::<E>(scope_manager, scope_id)?,
            Statement::Return(returns) => returns.desugar::<E>(scope_manager, scope_id)?,
        };

        if let Some(output) = output {
            *self = output;
        }

        Ok(None)
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
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Statement::Scope(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                Ok(())
            }
            Statement::Flow(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Statement::Assignation(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Statement::Declaration(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Statement::Definition(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Statement::Loops(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
            Statement::Return(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)
            }
        }
    }
}
