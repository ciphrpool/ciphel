use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{
    Atomic, BitwiseAnd, BitwiseOR, BitwiseXOR, Comparaison, Expression, HighOrdMath, LogicalAnd,
    LogicalOr, LowOrdMath, Shift, UnaryOperation,
};

impl Resolve for UnaryOperation {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for HighOrdMath {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for LowOrdMath {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for Shift {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for BitwiseAnd {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for BitwiseXOR {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for BitwiseOR {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for Comparaison {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for LogicalAnd {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl Resolve for LogicalOr {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
