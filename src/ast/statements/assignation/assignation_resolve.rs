use super::{AssignValue, Assignation};
use crate::semantic::scope::scope::Scope;
use crate::semantic::{ArcMutex, CompatibleWith, EType, Resolve, SemanticError, TypeOf};
use crate::vm::vm::Locatable;

impl Resolve for Assignation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.left.resolve(scope, &None, &mut None)?;
        if self.left.is_assignable() {
            let left_type = Some(self.left.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?);
            let _ = self.right.resolve(scope, &left_type, &mut ())?;
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
            AssignValue::Scope(value) => {
                let _ = value.resolve(scope, context, &mut Vec::default())?;
                let scope_type = value.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;

                match context {
                    Some(context) => {
                        let _ = context.compatible_with(&scope_type, &crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;
                    }
                    None => {}
                }
                Ok(())
            }
            AssignValue::Expr(value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let scope_type = value.type_of(&crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;
                match context {
                    Some(context) => {
                        let _ = context.compatible_with(&scope_type, &crate::arw_read!(scope,SemanticError::ConcurrencyError)?)?;
                    }
                    None => {}
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::TryParse,
        p_num,
        semantic::{
            scope::{
                scope::Scope,
                static_types::{NumberType, PrimitiveType, StaticType},
                var_impl::Var,
            },
            Either,
        },
    };

    use super::*;

    #[test]
    fn valid_assignation() {
        let mut assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = assignation.resolve(&scope, &None, &mut ());
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

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = assignation.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_assignation() {
        let mut assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = assignation.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }
}
