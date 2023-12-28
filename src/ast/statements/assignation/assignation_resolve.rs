use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{AssignValue, Assignation, Assignee};

impl<Scope: ScopeApi> Resolve<Scope> for Assignation {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope)?;
        let _ = self.right.resolve(scope)?;

        let left_type = self.left.type_of(scope)?;
        let _ = left_type.compatible_with(&self.right, scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for AssignValue {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            AssignValue::Scope(value) => value.resolve(scope),
            AssignValue::Expr(value) => value.resolve(scope),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Assignee {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Assignee::Variable(value) => value.resolve(scope),
            Assignee::PtrAccess(value) => value.resolve(scope),
        }
    }
}
