use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};

impl<Scope: ScopeApi> Resolve<Scope> for Loop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Loop::For(value) => value.resolve(scope),
            Loop::While(value) => value.resolve(scope),
            Loop::Loop(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForIterator {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ForIterator::Id(value) => {
                let _ = scope.find_var(value)?;
                // TODO : check that the variable is iterable

                Ok(())
            }
            ForIterator::Vec(value) => value.resolve(scope),
            ForIterator::Slice(value) => value.resolve(scope),
            ForIterator::Tuple(value) => value.resolve(scope),
            ForIterator::Receive { addr, .. } => addr.resolve(scope),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ForItem {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ForItem::Id(value) => Ok(()),
            ForItem::Pattern(_) => todo!(),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForLoop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.iterator.resolve(scope)?;
        let _ = self.item.resolve(scope)?;
        // TODO : attach the item to the scope
        let _ = self.scope.resolve(scope)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for WhileLoop {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope)?;
        // TODO check that the condition is a boolean
        let _ = self.scope.resolve(scope)?;
        Ok(())
    }
}
