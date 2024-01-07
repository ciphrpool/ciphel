use super::{AssignValue, Assignation, Assignee};
use crate::semantic::{
    scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for Assignation<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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

impl<Scope: ScopeApi> Resolve<Scope> for Assignee {
    type Output = ();
    type Context = ();
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
            Assignee::Variable(value) => value.resolve(scope, &None, extra),
            Assignee::PtrAccess(value) => value.resolve(scope, &None, extra),
        }
    }
}
