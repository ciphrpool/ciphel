use super::{AssignValue, Assignation, Assignee};
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
    CompatibleWith, EType, Either, MutRc, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for Assignation<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, &(), &())?;
        let left_type = Some(self.left.type_of(&scope.borrow())?);
        let _ = self.right.resolve(scope, &left_type, &())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for AssignValue<Scope> {
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
            AssignValue::Scope(value) => {
                let _ = value.resolve(scope, context, &Vec::default())?;
                let scope_type = value.type_of(&scope.borrow())?;

                match context {
                    Some(context) => {
                        let _ = context.compatible_with(&scope_type, &scope.borrow())?;
                    }
                    None => {}
                }
                Ok(())
            }
            AssignValue::Expr(value) => {
                let _ = value.resolve(scope, context, extra)?;
                let scope_type = value.type_of(&scope.borrow())?;
                match context {
                    Some(context) => {
                        let _ = context.compatible_with(&scope_type, &scope.borrow())?;
                    }
                    None => {}
                }
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Assignee<Scope> {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        _context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Assignee::Variable(value) => value.resolve(scope, &None, extra),
            Assignee::PtrAccess(value) => value.resolve(scope, &None, extra),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{NumberType, PrimitiveType, StaticType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_assignation() {
        let assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = assignation.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let assignation = Assignation::parse(
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
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = assignation.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_assignation() {
        let assignation = Assignation::parse("x = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = assignation.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}
