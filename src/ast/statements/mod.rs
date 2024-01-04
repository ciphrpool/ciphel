use nom::{
    branch::alt,
    combinator::{map, opt},
    sequence::delimited,
};

use super::{
    expressions::Expression,
    utils::{lexem, strings::wst},
    TryParse,
};
use crate::semantic::scope::BuildStaticType;
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf},
};

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
    Return(Return),
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
            map(Return::parse, |value| Statement::Return(value)),
        ))(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Statement {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Statement::Scope(value) => value.resolve(scope, context),
            Statement::Flow(value) => value.resolve(scope, context),
            Statement::Assignation(value) => value.resolve(scope, context),
            Statement::Declaration(value) => value.resolve(scope, context),
            Statement::Definition(value) => value.resolve(scope, context),
            Statement::Loops(value) => value.resolve(scope, context),
            Statement::Return(value) => value.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Statement {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Statement::Scope(value) => value.type_of(scope),
            Statement::Flow(value) => value.type_of(scope),
            Statement::Assignation(value) => value.type_of(scope),
            Statement::Declaration(value) => value.type_of(scope),
            Statement::Definition(value) => Ok(EitherType::Static(Scope::StaticType::build_unit())),
            Statement::Loops(value) => value.type_of(scope),
            Statement::Return(value) => value.type_of(scope),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Unit,
    Expr(Box<Expression>),
}

impl TryParse for Return {
    /*
     * @desc Parse return statements
     *
     * @grammar
     * Return := return
     *      | ID
     *      | Expr
     *      | Λ
     */
    fn parse(input: Span) -> PResult<Self> {
        map(
            delimited(
                wst(lexem::RETURN),
                opt(Expression::parse),
                wst(lexem::SEMI_COLON),
            ),
            |value| match value {
                Some(expr) => Return::Expr(Box::new(expr)),
                None => Return::Unit,
            },
        )(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Return {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Return::Unit => Ok(()),
            Return::Expr(value) => value.resolve(scope, &None),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Return {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Return::Unit => Ok(EitherType::Static(Scope::StaticType::build_unit())),
            Return::Expr(expr) => expr.type_of(scope),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_return() {
        let res = Return::parse(
            r#"
            return ;
        "#
            .into(),
        );
        assert!(res.is_ok());
        let value = res.unwrap().1;
        assert_eq!(Return::Unit, value);
    }
}
