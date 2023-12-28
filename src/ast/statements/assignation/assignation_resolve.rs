use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{AssignValue, Assignation, Assignee};

impl Resolve for Assignation {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
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
impl Resolve for AssignValue {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}

impl Resolve for Assignee {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
