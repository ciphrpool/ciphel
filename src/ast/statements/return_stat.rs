use std::cell::Ref;

use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::{AccessLevel, Info};
use crate::vm::allocator::stack::{Offset, UReg};
use crate::vm::allocator::MemoryAddress;
use crate::vm::casm::alloc::Access;
use crate::vm::casm::branch::Goto;
use crate::vm::casm::mem::Mem;
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
        CompatibleWith, EType, Either, Metadata, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
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
    Yield {
        expr: Box<Expression>,
        metadata: Metadata,
    },
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
            map(
                delimited(wst(lexem::YIELD), Expression::parse, wst(lexem::SEMI_COLON)),
                |e| Return::Yield {
                    expr: Box::new(e),
                    metadata: Metadata::default(),
                },
            ),
        ))(input)
    }
}
impl Resolve for Return {
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
            Return::Break => {
                if scope.as_ref().borrow().state().is_loop {
                    Ok(())
                } else {
                    Err(SemanticError::ExpectedLoop)
                }
            }
            Return::Continue => {
                if scope.as_ref().borrow().state().is_loop {
                    Ok(())
                } else {
                    Err(SemanticError::ExpectedLoop)
                }
            }
            Return::Yield { expr, metadata } => {
                let _ = expr.resolve(scope, &context, extra)?;
                if scope.as_ref().borrow().state().is_generator {
                    resolve_metadata!(metadata, self, scope, context);
                    Ok(())
                } else {
                    Err(SemanticError::ExpectedLoop)
                }
            }
        }
    }
}

impl TypeOf for Return {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Return::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Return::Expr { expr, metadata } => expr.type_of(&scope),
            Return::Break => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Return::Continue => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Return::Yield { expr, metadata } => expr.type_of(&scope),
        }
    }
}

impl GenerateCode for Return {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
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
            Return::Yield { expr, metadata } => {
                let Some(return_size) = metadata.signature().map(|t| t.size_of()) else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let _ = expr.gencode(scope, instructions)?;

                instructions.push(Casm::StackFrame(StackFrame::Yield {
                    return_size: Some(return_size),
                }));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        e_static, p_num,
        semantic::{
            scope::{
                scope_impl::{self},
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
        let return_statement = Return::parse(
            r#"
            return ;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();

        let res = return_statement.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let return_statement = Return::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let res = return_statement.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(p_num!(I64), return_type);

        let return_statement = Return::parse(
            r#"
            break ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let inner_scope = scope_impl::Scope::spawn(&scope, Vec::default())
            .expect("Scope should be able to have child block");
        inner_scope.as_ref().borrow_mut().to_loop();

        let res = return_statement.resolve(&inner_scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(e_static!(StaticType::Unit), return_type);

        let return_statement = Return::parse(
            r#"
            continue ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let res = return_statement.resolve(&inner_scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(e_static!(StaticType::Unit), return_type);

        let return_statement = Return::parse(
            r#"
            yield 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        inner_scope.as_ref().borrow_mut().to_generator();
        let res = return_statement.resolve(&inner_scope, &Some(p_num!(U64)), &());
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope.borrow()).unwrap();
        assert_eq!(p_num!(U64), return_type);
    }

    #[test]
    fn robustness_return() {
        let return_statement = Return::parse(
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
