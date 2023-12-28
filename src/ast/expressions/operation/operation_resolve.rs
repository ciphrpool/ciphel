use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{
    Atomic, BitwiseAnd, BitwiseOR, BitwiseXOR, Comparaison, Expression, HighOrdMath, LogicalAnd,
    LogicalOr, LowOrdMath, Shift, UnaryOperation,
};

impl<Scope: ScopeApi> Resolve<Scope> for UnaryOperation {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for HighOrdMath {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LowOrdMath {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Shift {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseAnd {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseXOR {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseOR {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Comparaison {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalAnd {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalOr {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
