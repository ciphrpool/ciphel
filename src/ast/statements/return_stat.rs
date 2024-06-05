use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
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
        scope::{static_types::StaticType, type_traits::TypeChecking, BuildStaticType},
        ArcMutex, CompatibleWith, EType, Either, Metadata, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        casm::{alloc::StackFrame, Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};
use nom::branch::alt;
use nom::sequence::terminated;
use nom::{
    combinator::{map, opt},
    sequence::delimited,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Unit,
    Expr {
        expr: Box<Expression>,
        metadata: Metadata,
    },
    Break,
    Continue,
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
        alt((
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
            ),
            map(
                terminated(wst(lexem::BREAK), wst(lexem::SEMI_COLON)),
                |_| Return::Break,
            ),
            map(
                terminated(wst(lexem::CONTINUE), wst(lexem::SEMI_COLON)),
                |_| Return::Continue,
            ),
        ))(input)
    }
}
impl Resolve for Return {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
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
                let _ = expr.resolve(scope, &context, &mut None)?;
                match context {
                    Some(c) => {
                        let return_type = expr
                            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                        let _ = c.compatible_with(
                            &return_type,
                            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                        )?;
                        metadata.info = Info::Resolved {
                            context: context.clone(),
                            signature: Some(expr.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?),
                        };
                        Ok(())
                    }
                    None => {
                        metadata.info = Info::Resolved {
                            context: None,
                            signature: Some(expr.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?),
                        };
                        Ok(())
                    }
                }
            }
            Return::Break => {
                if crate::arw_read!(scope, SemanticError::ConcurrencyError)?
                    .state()?
                    .is_loop
                {
                    Ok(())
                } else {
                    Err(SemanticError::ExpectedLoop)
                }
            }
            Return::Continue => {
                if crate::arw_read!(scope, SemanticError::ConcurrencyError)?
                    .state()?
                    .is_loop
                {
                    Ok(())
                } else {
                    Err(SemanticError::ExpectedLoop)
                }
            }
        }
    }
}

impl TypeOf for Return {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Return::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Return::Expr { expr, metadata: _ } => expr.type_of(&scope),
            Return::Break => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Return::Continue => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
        }
    }
}

impl GenerateCode for Return {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Return::Unit => {
                instructions.push(Casm::StackFrame(StackFrame::Return {
                    return_size: Some(0),
                }));
                Ok(())
            }
            Return::Expr { expr, metadata } => {
                let Some(return_size) = metadata.signature().map(|t| t.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = expr.gencode(scope, instructions)?;

                instructions.push(Casm::StackFrame(StackFrame::Return {
                    return_size: Some(return_size),
                }));
                Ok(())
            }
            Return::Break => {
                instructions.push(Casm::StackFrame(StackFrame::Break));
                Ok(())
            }
            Return::Continue => {
                instructions.push(Casm::StackFrame(StackFrame::Continue));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arw_write, e_static, p_num,
        semantic::{
            scope::{
                scope,
                static_types::{NumberType, PrimitiveType, StaticType},
            },
            Either,
        },
    };

    use super::*;

    #[test]
    fn valid_return() {
        let res = Return::parse(
            r#"
            return ;
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Return::Unit, value);

        let res = Return::parse(
            r#"
            break ;
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Return::Break, value);
        let res = Return::parse(
            r#"
            continue ;
        "#
            .into(),
        );
        assert!(res.is_ok(), "{:?}", res);
        let value = res.unwrap().1;
        assert_eq!(Return::Continue, value);
    }

    #[test]
    fn valid_resolved_return() {
        let mut return_statement = Return::parse(
            r#"
            return ;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope::Scope::new();

        let res = return_statement.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut return_statement = Return::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let res = return_statement.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(I64), return_type);

        let mut return_statement = Return::parse(
            r#"
            break ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let inner_scope = scope::Scope::spawn(&scope, Vec::default())
            .expect("Scope should be able to have child block");
        arw_write!(inner_scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .to_loop();

        let res = return_statement.resolve(&inner_scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(e_static!(StaticType::Unit), return_type);

        let mut return_statement = Return::parse(
            r#"
            continue ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let res = return_statement.resolve(&inner_scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(e_static!(StaticType::Unit), return_type);
    }

    #[test]
    fn robustness_return() {
        let mut return_statement = Return::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope::Scope::new();
        let res = return_statement.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Char).into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }
}
