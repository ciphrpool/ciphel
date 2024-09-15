use super::{AssignValue, Assignation};
use crate::ast::expressions::locate::Locatable;
use crate::semantic::{CompatibleWith, EType, Resolve, SemanticError, TypeOf};

impl Resolve for Assignation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
        if self.left.is_assignable() {
            let left_type = self.left.type_of(&scope_manager, scope_id)?;
            let _ = self
                .right
                .resolve::<G>(scope_manager, scope_id, &Some(left_type), &mut ())?;

            Ok(())
        } else {
            Err(SemanticError::ExpectedLeftExpression)
        }
    }
}
impl Resolve for AssignValue {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut ())?;
                let scope_type = value.type_of(&scope_manager, scope_id)?;

                if let Some(context) = context {
                    let _ = context.compatible_with(&scope_type, &scope_manager, scope_id)?;
                }
                Ok(())
            }
            AssignValue::Expr(value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let scope_type = value.type_of(&scope_manager, scope_id)?;

                if let Some(context) = context {
                    let _ = context.compatible_with(&scope_type, &scope_manager, scope_id)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::TryParse, p_num, semantic::scope::scope::ScopeManager};

    use super::*;

    #[test]
    fn valid_assignation() {
        let mut assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = assignation.resolve::<crate::vm::vm::NoopGameEngine>(
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
                return y;
            };
        "##
            .into(),
        )
        .unwrap()
        .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = assignation.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = assignation.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }
}
