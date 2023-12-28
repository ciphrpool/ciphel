use nom::{branch::alt, combinator::map};

use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{EitherType, Resolve, ScopeApi, SemanticError},
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
            // map(TryStat::parse, |value| Statement::Try(value)),
            // map(CallStat::parse, |value| Statement::Call(value)),
            // map(Return::parse, |value| Statement::Return(value)),
        ))(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Statement {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
