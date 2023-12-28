use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

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
        // TODO : check if type is compatible with the unary operation
        match self {
            UnaryOperation::Minus(value) => {
                let _ = value.resolve(scope)?;
                Ok(())
            }
            UnaryOperation::Not(value) => {
                let _ = value.resolve(scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for HighOrdMath {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        // TODO : Check if type is compatible with the operation
        match self {
            HighOrdMath::Mult { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            HighOrdMath::Div { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            HighOrdMath::Mod { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LowOrdMath {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            LowOrdMath::Add { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Shift {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Shift::Left { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Shift::Right { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseAnd {
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
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseXOR {
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
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseOR {
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
impl<Scope: ScopeApi> Resolve<Scope> for Comparaison {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Comparaison::Less { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::LessEqual { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::Greater { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::GreaterEqual { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::Equal { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::NotEqual { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
            Comparaison::In { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
                let _ = left_type.compatible_with(right, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalAnd {
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
impl<Scope: ScopeApi> Resolve<Scope> for LogicalOr {
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
