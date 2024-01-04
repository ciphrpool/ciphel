use crate::semantic::{
    scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};

use super::{AssignValue, Assignation, Assignee};

impl<Scope: ScopeApi> Resolve<Scope> for Assignation {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, &())?;
        let left_type = Some(self.left.type_of(scope)?);
        let _ = self.right.resolve(scope, &left_type)?;

        let _ = left_type.compatible_with(&self.right, scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for AssignValue {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            AssignValue::Scope(value) => value.resolve(scope, context),
            AssignValue::Expr(value) => value.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Assignee {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Assignee::Variable(value) => value.resolve(scope, &None),
            Assignee::PtrAccess(value) => value.resolve(scope, &None),
        }
    }
}
