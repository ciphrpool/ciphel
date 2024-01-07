use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

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
use crate::semantic::{scope::BuildStaticType};
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
pub enum Statement<InnerScope: ScopeApi> {
    Scope(scope::Scope<InnerScope>),
    Flow(flows::Flow<InnerScope>),
    Assignation(assignation::Assignation<InnerScope>),
    Declaration(declaration::Declaration<InnerScope>),
    Definition(definition::Definition<InnerScope>),
    Loops(loops::Loop<InnerScope>),
    Return(Return<InnerScope>),
}

impl<InnerScope: ScopeApi> TryParse for Statement<InnerScope> {
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
impl<Scope: ScopeApi> Resolve<Scope> for Statement<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Statement::Scope(value) => {
                let _ = value.resolve(scope, context, &Vec::default())?;
                Ok(())
            }
            Statement::Flow(value) => value.resolve(scope, context, extra),
            Statement::Assignation(value) => value.resolve(scope, context, extra),
            Statement::Declaration(value) => value.resolve(scope, context, extra),
            Statement::Definition(value) => value.resolve(scope, context, extra),
            Statement::Loops(value) => value.resolve(scope, context, extra),
            Statement::Return(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Statement<Scope> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope),
            Statement::Flow(value) => value.type_of(&scope),
            Statement::Assignation(value) => value.type_of(&scope),
            Statement::Declaration(value) => value.type_of(&scope),
            Statement::Definition(_value) => {
                Ok(EitherType::Static(Scope::StaticType::build_unit()))
            }
            Statement::Loops(value) => value.type_of(&scope),
            Statement::Return(value) => value.type_of(&scope),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Return<InnerScope: ScopeApi> {
    Unit,
    Expr(Box<Expression<InnerScope>>),
}

impl<Scope: ScopeApi> TryParse for Return<Scope> {
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
impl<Scope: ScopeApi> Resolve<Scope> for Return<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Return::Unit => Ok(()),
            Return::Expr(value) => value.resolve(scope, &None, extra),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Return<Scope> {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Return::Unit => Ok(EitherType::Static(Scope::StaticType::build_unit())),
            Return::Expr(expr) => expr.type_of(&scope),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::scope::scope_impl::{MockScope};

    use super::*;

    #[test]
    fn valid_return() {
        let res = Return::<MockScope>::parse(
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
