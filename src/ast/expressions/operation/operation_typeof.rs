use std::cell::Ref;

use crate::semantic::{
    scope::{type_traits::OperandMerging, ScopeApi},
    EitherType, Resolve, SemanticError, TypeOf,
};

use super::{
    BitwiseAnd, BitwiseOR, BitwiseXOR, Comparaison, HighOrdMath, LogicalAnd, LogicalOr, LowOrdMath,
    Shift, UnaryOperation,
};

impl<Scope: ScopeApi> TypeOf<Scope> for UnaryOperation {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            UnaryOperation::Minus(value) => value.type_of(&scope),
            UnaryOperation::Not(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for HighOrdMath {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            HighOrdMath::Mult { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_high_ord_math(&right_type, scope)
            }
            HighOrdMath::Div { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_high_ord_math(&right_type, scope)
            }
            HighOrdMath::Mod { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_high_ord_math(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for LowOrdMath {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            LowOrdMath::Add { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_low_ord_math(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Shift {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Shift::Left { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_shift(&right_type, scope)
            }
            Shift::Right { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_shift(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseAnd {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_and(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseXOR {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_xor(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseOR {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_or(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Comparaison {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Comparaison::Less { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::LessEqual { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::Greater { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::GreaterEqual { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::Equal { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::NotEqual { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::In { left, right } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for LogicalAnd {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_and(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for LogicalOr {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_or(&right_type, scope)
    }
}
