use std::cell::Ref;

use crate::semantic::{
    scope::{
        static_types::{self, StaticType},
        type_traits::OperandMerging,
        user_type_impl::UserType,
        BuildStaticType, ScopeApi,
    },
    EType, Either, MergeType, Resolve, SemanticError, TypeOf,
};

use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, Inclusion,
    LogicalAnd, LogicalOr, Product, Range, Shift, Substraction, UnaryOperation,
};

impl<Scope: ScopeApi> TypeOf<Scope> for UnaryOperation<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            UnaryOperation::Minus { value, .. } => value.type_of(&scope),
            UnaryOperation::Not { value, .. } => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Range<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let type_sig = match self
            .lower
            .type_of(scope)?
            .merge(self.upper.as_ref(), scope)?
        {
            Either::Static(value) => match value.as_ref() {
                StaticType::Primitive(static_types::PrimitiveType::Number(value)) => value.clone(),
                _ => return Err(SemanticError::IncompatibleTypes),
            },
            Either::User(_) => return Err(SemanticError::IncompatibleTypes),
        };
        StaticType::build_range_from(type_sig, self.inclusive, scope)
            .map(|value| Either::Static(value.into()))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Product<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Product::Mult {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_product(&right_type, scope)
            }
            Product::Div {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_product(&right_type, scope)
            }
            Product::Mod {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_product(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Addition<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_addition(&right_type, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Substraction<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_substraction(&right_type, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Shift<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Shift::Left {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_shift(&right_type, scope)
            }
            Shift::Right {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_shift(&right_type, scope)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseAnd<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_and(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseXOR<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_xor(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for BitwiseOR<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_or(&right_type, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Cast<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.cast(&right_type, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Comparaison<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Comparaison::Less {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::LessEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::Greater {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
            Comparaison::GreaterEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_comparaison(&right_type, scope)
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Equation<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Equation::Equal {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_equation(&right_type, scope)
            }
            Equation::NotEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope)?;
                let right_type = right.type_of(&scope)?;
                left_type.merge_equation(&right_type, scope)
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Inclusion<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_inclusion(&right_type, scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for LogicalAnd<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_and(&right_type, scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for LogicalOr<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_or(&right_type, scope)
    }
}
