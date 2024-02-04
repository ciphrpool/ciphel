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
use crate::{
    ast::utils::io::{PResult, Span},
    semantic::{
        scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
        Either, Resolve, SemanticError, TypeOf,
    },
    vm::{strips::Strip, vm::CodeGenerationError},
};
use crate::{
    semantic::{
        scope::{type_traits::TypeChecking, BuildStaticType},
        CompatibleWith,
    },
    vm::vm::GenerateCode,
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
    type Context = Option<Either<UserType, StaticType>>;
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
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Statement::Scope(value) => value.type_of(&scope),
            Statement::Flow(value) => value.type_of(&scope),
            Statement::Assignation(value) => value.type_of(&scope),
            Statement::Declaration(value) => value.type_of(&scope),
            Statement::Definition(_value) => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Statement::Loops(value) => value.type_of(&scope),
            Statement::Return(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Statement<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Statement::Scope(value) => value.gencode(scope, instructions, offset),
            Statement::Flow(value) => value.gencode(scope, instructions, offset),
            Statement::Assignation(value) => value.gencode(scope, instructions, offset),
            Statement::Declaration(value) => value.gencode(scope, instructions, offset),
            Statement::Definition(value) => value.gencode(scope, instructions, offset),
            Statement::Loops(value) => value.gencode(scope, instructions, offset),
            Statement::Return(_) => todo!(),
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
    type Context = Option<Either<UserType, StaticType>>;
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
            Return::Unit => {
                match context {
                    Some(context) => {
                        if !<Either<UserType, StaticType> as TypeChecking>::is_unit(context) {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                    }
                    None => {}
                }
                Ok(())
            }
            Return::Expr(value) => {
                let _ = value.resolve(scope, &context, extra)?;
                match context {
                    Some(context) => {
                        let return_type = value.type_of(&scope.borrow())?;
                        let _ = context.compatible_with(&return_type, &scope.borrow())?;
                        Ok(())
                    }
                    None => Ok(()),
                }
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Return<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Return::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Return::Expr(expr) => expr.type_of(&scope),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::scope::{
        scope_impl::{self, MockScope},
        static_types::{NumberType, PrimitiveType, StaticType},
    };

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

    #[test]
    fn valid_resolved_return() {
        let return_statement = Return::<scope_impl::Scope>::parse(
            r#"
            return ;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = return_statement.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(Either::Static(StaticType::Unit.into()), return_type);

        let return_statement = Return::<scope_impl::Scope>::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = return_statement.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            return_type
        );
    }

    #[test]
    fn robustness_return() {
        let return_statement = Return::<scope_impl::Scope>::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = return_statement.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Char).into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }
}
