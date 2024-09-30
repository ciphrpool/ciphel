use super::{AssignValue, Assignation};
use crate::ast::expressions::locate::Locatable;
use crate::ast::statements::Statement;
use crate::semantic::{CompatibleWith, Desugar, EType, Resolve, SemanticError, TypeOf};

impl Resolve for Assignation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .left
            .resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        if self.left.is_assignable() {
            let left_type = self.left.type_of(&scope_manager, scope_id)?;
            let _ = self
                .right
                .resolve::<E>(scope_manager, scope_id, &Some(left_type), &mut ())?;

            Ok(())
        } else {
            Err(SemanticError::ExpectedLeftExpression)
        }
    }
}

impl Desugar<Statement> for Assignation {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            self.left = output.into();
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            self.right = output.into();
        }
        Ok(None)
    }
}

impl Resolve for AssignValue {
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
        match self {
            AssignValue::Block(value) => {
                let _ = value.resolve::<E>(scope_manager, scope_id, context, &mut ())?;
                let scope_type = value.type_of(&scope_manager, scope_id)?;

                if let Some(context) = context {
                    let _ = context.compatible_with(&scope_type, &scope_manager, scope_id)?;
                }
                Ok(())
            }
            AssignValue::Expr(value) => {
                let _ = value.resolve::<E>(scope_manager, scope_id, context, &mut None)?;
                let scope_type = value.type_of(&scope_manager, scope_id)?;

                if let Some(context) = context {
                    let _ = context.compatible_with(&scope_type, &scope_manager, scope_id)?;
                }
                Ok(())
            }
        }
    }
}

impl Desugar<AssignValue> for AssignValue {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<AssignValue>, SemanticError> {
        match self {
            AssignValue::Block(expr_block) => {
                if let Some(output) = expr_block.desugar::<E>(scope_manager, scope_id)? {
                    *expr_block = output.into();
                }
            }
            AssignValue::Expr(expression) => {
                if let Some(output) = expression.desugar::<E>(scope_manager, scope_id)? {
                    *expression = output.into();
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::TryParse, p_num};

    use super::*;

    #[test]
    fn valid_assignation() {
        let mut assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = assignation.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut assignation = Assignation::parse(
            r##"
            x = { 
                let y = 10;
                y
            };
        "##
            .into(),
        )
        .unwrap()
        .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = assignation.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_assignation() {
        let mut assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = assignation.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }
}
