use crate::ast::utils::error::squash;
use crate::ast::utils::strings::wst_closed;
use crate::semantic::scope::scope::{ScopeManager, ScopeState};
use crate::semantic::{Desugar, Info};

use crate::vm::asm::branch::Goto;
use crate::vm::{CodeGenerationError, GenerateCode};
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
    semantic::{
        scope::static_types::StaticType, CompatibleWith, EType, Metadata, Resolve, SemanticError,
        SizeOf, TypeOf,
    },
    vm::asm::Asm,
};
use nom::branch::alt;
use nom::combinator::cut;
use nom::sequence::terminated;
use nom::{
    combinator::{map, opt},
    sequence::delimited,
};
use nom_supreme::context;

use super::Statement;

#[derive(Debug, Clone, PartialEq)]
pub enum Return {
    Unit,
    Expr {
        expr: Box<Expression>,
        metadata: Metadata,
    },
    Inline {
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
        squash(
            alt((
                map(
                    delimited(
                        wst_closed(lexem::RETURN),
                        cut(opt(Expression::parse)),
                        cut(wst(lexem::SEMI_COLON)),
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
                    terminated(wst_closed(lexem::BREAK), cut(wst(lexem::SEMI_COLON))),
                    |_| Return::Break,
                ),
                map(
                    terminated(wst_closed(lexem::CONTINUE), cut(wst(lexem::SEMI_COLON))),
                    |_| Return::Continue,
                ),
            )),
            "Expected a return, break or continue statement",
        )(input)
    }
}

impl Desugar<Statement> for Return {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        match self {
            Return::Unit => {}
            Return::Expr { expr, metadata } => {
                if let Some(output) = expr.desugar::<E>(scope_manager, scope_id)? {
                    *expr = output.into();
                }
            }
            Return::Inline { expr, metadata } => {
                if let Some(output) = expr.desugar::<E>(scope_manager, scope_id)? {
                    *expr = output.into();
                }
            }
            Return::Break => {}
            Return::Continue => {}
        }
        return Ok(None);
    }
}

impl Resolve for Return {
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
        let Some(scope_id) = scope_id else {
            return Err(SemanticError::Default);
        };
        match self {
            Return::Unit => {
                let Some(scope_id) = scope_manager.can_return(scope_id) else {
                    return Err(SemanticError::ExpectedFunctionClosureLambda);
                };
                match scope_manager.scope_types.get_mut(&scope_id) {
                    Some(types) => {
                        types.push(EType::Static(StaticType::Unit));
                    }
                    None => {
                        scope_manager
                            .scope_types
                            .insert(scope_id, vec![EType::Static(StaticType::Unit)]);
                    }
                }
                if let Some(context) = context {
                    if *context != EType::Static(StaticType::Unit) {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
                Ok(())
            }
            Return::Expr { expr, metadata } => {
                let _ = expr.resolve::<E>(scope_manager, Some(scope_id), &context, &mut None)?;
                let return_type = expr.type_of(&scope_manager, Some(scope_id))?;
                if let Some(context) = context {
                    let _ =
                        context.compatible_with(&return_type, &scope_manager, Some(scope_id))?;
                }

                let Some(scope_id) = scope_manager.can_return(scope_id) else {
                    return Err(SemanticError::ExpectedFunctionClosureLambda);
                };
                match scope_manager.scope_types.get_mut(&scope_id) {
                    Some(types) => {
                        types.push(return_type.clone());
                    }
                    None => {
                        scope_manager
                            .scope_types
                            .insert(scope_id, vec![return_type.clone()]);
                    }
                }
                if let Some(context) = context {
                    if *context != return_type {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(return_type),
                };
                Ok(())
            }
            Return::Inline { expr, metadata } => {
                let _ = expr.resolve::<E>(scope_manager, Some(scope_id), &context, &mut None)?;
                let return_type = expr.type_of(&scope_manager, Some(scope_id))?;
                if let Some(context) = context {
                    let _ =
                        context.compatible_with(&return_type, &scope_manager, Some(scope_id))?;
                }

                let scope_id = match scope_manager.can_return(scope_id) {
                    Some(scope_id) => scope_id,
                    None => match scope_manager.scope_states.get(&scope_id) {
                        Some(ScopeState::Inline | ScopeState::IIFE) => scope_id,
                        _ => return Err(SemanticError::ExpectedInlineBlock),
                    },
                };
                match scope_manager.scope_types.get_mut(&scope_id) {
                    Some(types) => {
                        types.push(return_type.clone());
                    }
                    None => {
                        scope_manager
                            .scope_types
                            .insert(scope_id, vec![return_type.clone()]);
                    }
                }

                if let Some(context) = context {
                    if *context != return_type {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(return_type),
                };
                Ok(())
            }
            Return::Break => {
                let Some(scope_id) = scope_manager.is_scope_in(scope_id, ScopeState::Loop) else {
                    return Err(SemanticError::ExpectedLoop);
                };
                Ok(())
            }
            Return::Continue => {
                let Some(scope_id) = scope_manager.is_scope_in(scope_id, ScopeState::Loop) else {
                    return Err(SemanticError::ExpectedLoop);
                };
                Ok(())
            }
        }
    }
}

impl TypeOf for Return {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Return::Unit => Ok(EType::Static(StaticType::Unit)),
            Return::Expr { expr, metadata: _ } => expr.type_of(&scope_manager, scope_id),
            Return::Inline { expr, metadata: _ } => expr.type_of(&scope_manager, scope_id),
            Return::Break => Ok(EType::Static(StaticType::Unit)),
            Return::Continue => Ok(EType::Static(StaticType::Unit)),
        }
    }
}

impl GenerateCode for Return {
    fn gencode<E:crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            Return::Unit => {
                let Some(return_label) = context.return_label else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Goto(Goto {
                    label: Some(return_label),
                }));
                Ok(())
            }
            Return::Expr { expr, metadata } => {
                let _ = expr.gencode::<E>(scope_manager, scope_id, instructions, context)?;

                let Some(return_label) = context.return_label else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Goto(Goto {
                    label: Some(return_label),
                }));
                Ok(())
            }
            Return::Inline { expr, metadata } => {
                let _ = expr.gencode::<E>(scope_manager, scope_id, instructions, context)?;

                let Some(return_label) = context.return_label else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Goto(Goto {
                    label: Some(return_label),
                }));
                Ok(())
            }
            Return::Break => {
                let Some(break_label) = context.break_label else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Goto(Goto {
                    label: Some(break_label),
                }));
                Ok(())
            }
            Return::Continue => {
                let Some(continue_label) = context.continue_label else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Goto(Goto {
                    label: Some(continue_label),
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
                scope,
                static_types::{PrimitiveType, StaticType},
            },
            EType,
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
        let mut scope_manager = scope::ScopeManager::default();

        let res = return_statement.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut return_statement = Return::parse(
            r#"
            return 10;
        "#
            .into(),
        )
        .unwrap()
        .1;
        let res = return_statement.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope_manager, None).unwrap();
        assert_eq!(p_num!(I64), return_type);

        let mut return_statement = Return::parse(
            r#"
            break ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let inner_scope = scope_manager
            .spawn(None)
            .expect("Scope should be able to have child block");
        scope_manager
            .scope_states
            .insert(inner_scope, ScopeState::Loop);
        let res = return_statement.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            Some(inner_scope),
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope_manager, None).unwrap();
        assert_eq!(e_static!(StaticType::Unit), return_type);

        let mut return_statement = Return::parse(
            r#"
            continue ;
        "#
            .into(),
        )
        .unwrap()
        .1;

        let res = return_statement.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            Some(inner_scope),
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let return_type = return_statement.type_of(&scope_manager, None).unwrap();
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
        let mut scope_manager = scope::ScopeManager::default();
        let res = return_statement.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Primitive(PrimitiveType::Char).into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }
}
