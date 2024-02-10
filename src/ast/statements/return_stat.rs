use std::cell::Ref;

use crate::semantic::Info;
use crate::{
    ast::{
        expressions::Expression,
        utils::{
            io::{PResult, Span},
            lexem,
            strings::wst,
        },
        TryParse,
    },
    resolve_metadata,
    semantic::{
        scope::{static_types::StaticType, type_traits::TypeChecking, BuildStaticType, ScopeApi},
        CompatibleWith, EType, Either, Metadata, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        casm::{alloc::StackFrame, Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};
use nom::{
    combinator::{map, opt},
    sequence::delimited,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Return<InnerScope: ScopeApi> {
    Unit,
    Expr {
        expr: Box<Expression<InnerScope>>,
        metadata: Metadata,
    },
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
                Some(expr) => Return::Expr {
                    expr: Box::new(expr),
                    metadata: Metadata::default(),
                },
                None => Return::Unit,
            },
        )(input)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Return<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
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
                        if !<EType as TypeChecking>::is_unit(context) {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                    }
                    None => {}
                }
                Ok(())
            }
            Return::Expr { expr, metadata } => {
                let _ = expr.resolve(scope, &context, extra)?;
                match context {
                    Some(c) => {
                        let return_type = expr.type_of(&scope.borrow())?;
                        let _ = c.compatible_with(&return_type, &scope.borrow())?;
                        resolve_metadata!(metadata, self, scope, context);
                        Ok(())
                    }
                    None => {
                        resolve_metadata!(metadata, self, scope, None);
                        Ok(())
                    }
                }
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Return<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Return::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Return::Expr { expr, metadata } => expr.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for Return<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Return::Unit => {
                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::StackFrame(StackFrame::Return { return_size: 0 }));
                Ok(())
            }
            Return::Expr { expr, metadata } => {
                let Some(return_size) = metadata.signature().map(|t| t.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = expr.gencode(scope, instructions)?;
                let mut borrowed = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                borrowed.push(Casm::StackFrame(StackFrame::Return { return_size }));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::{
        scope::{
            scope_impl::{self, MockScope},
            static_types::{NumberType, PrimitiveType, StaticType},
        },
        Either,
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
