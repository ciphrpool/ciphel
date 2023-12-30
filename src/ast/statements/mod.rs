use nom::{branch::alt, combinator::map};

use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf},
};

use super::TryParse;

pub mod assignation;
pub mod declaration;
pub mod definition;
pub mod flows;
pub mod loops;
pub mod scope;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Scope(scope::Scope),
    Flow(flows::Flow),
    Assignation(assignation::Assignation),
    Declaration(declaration::Declaration),
    Definition(definition::Definition),
    Loops(loops::Loop),
}

impl TryParse for Statement {
    fn parse(input: Span) -> PResult<Self> {
        alt((
            map(scope::Scope::parse, |value| Statement::Scope(value)),
            map(flows::Flow::parse, |value| Statement::Flow(value)),
            map(assignation::Assignation::parse, |value| {
                Statement::Assignation(value)
            }),
            map(declaration::Declaration::parse, |value| {
                Statement::Declaration(value)
            }),
            map(definition::Definition::parse, |value| {
                Statement::Definition(value)
            }),
            map(loops::Loop::parse, |value| Statement::Loops(value)),
        ))(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Statement {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Statement::Scope(value) => value.resolve(scope, &None),
            Statement::Flow(value) => value.resolve(scope, context),
            Statement::Assignation(value) => value.resolve(scope, context),
            Statement::Declaration(value) => value.resolve(scope, context),
            Statement::Definition(value) => value.resolve(scope, context),
            Statement::Loops(value) => value.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Statement {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Statement::Scope(value) => value.type_of(scope),
            Statement::Flow(value) => value.type_of(scope),
            Statement::Assignation(value) => value.type_of(scope),
            Statement::Declaration(value) => value.type_of(scope),
            Statement::Definition(value) => Ok(None),
            Statement::Loops(value) => value.type_of(scope),
        }
    }
}
