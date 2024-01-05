use super::{
    BitwiseAnd, BitwiseOR, BitwiseXOR, Comparaison, HighOrdMath, LogicalAnd, LogicalOr, LowOrdMath,
    Shift, UnaryOperation,
};
use crate::semantic::{
    scope::{type_traits::OperandMerging, ScopeApi},
    CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for UnaryOperation {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            UnaryOperation::Minus(value) => {
                let _ = value.resolve(scope, context)?;
                let value_type = value.type_of(&scope.borrow())?;
                let _ = value_type.can_minus()?;
                Ok(())
            }
            UnaryOperation::Not(value) => {
                let _ = value.resolve(scope, context)?;
                let value_type = value.type_of(&scope.borrow())?;
                let _ = value_type.can_negate()?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for HighOrdMath {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            HighOrdMath::Mult { left, right } => (left, right),
            HighOrdMath::Div { left, right } => (left, right),
            HighOrdMath::Mod { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_high_ord_math()?;

        let _ = right.resolve(scope, context)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_high_ord_math()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LowOrdMath {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            LowOrdMath::Add { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_low_ord_math()?;

        let _ = right.resolve(scope, context)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_low_ord_math()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Shift {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            Shift::Left { left, right } => (left, right),
            Shift::Right { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_shift()?;

        let _ = right.resolve(scope, context)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_shift()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseAnd {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_and()?;

        let _ = self.right.resolve(scope, context)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_and()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseXOR {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_xor()?;

        let _ = self.right.resolve(scope, context)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_xor()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseOR {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_or()?;

        let _ = self.right.resolve(scope, context)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_or()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Comparaison {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            Comparaison::Less { left, right } => (left, right),
            Comparaison::LessEqual { left, right } => (left, right),
            Comparaison::Greater { left, right } => (left, right),
            Comparaison::GreaterEqual { left, right } => (left, right),
            Comparaison::Equal { left, right } => (left, right),
            Comparaison::NotEqual { left, right } => (left, right),
            Comparaison::In { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_comparaison()?;

        let _ = right.resolve(scope, context)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_comparaison()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalAnd {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_logical_and()?;

        let _ = self.right.resolve(scope, context)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_logical_and()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalOr {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_logical_or()?;

        let _ = self.right.resolve(scope, context)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_logical_or()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
