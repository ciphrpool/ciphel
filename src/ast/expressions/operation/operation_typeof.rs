use std::cell::Ref;

use crate::{
    e_static,
    semantic::{
        scope::{
            static_types::{self, StaticType},
            type_traits::OperandMerging,
            BuildStaticType,
        },
        EType, Either, MergeType, Resolve, SemanticError, TypeOf,
    },
};

use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, LogicalAnd,
    LogicalOr, Product, Range, Shift, Substraction, UnaryOperation,
};
use crate::semantic::scope::scope_impl::Scope;

impl TypeOf for UnaryOperation {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            UnaryOperation::Minus { value, .. } => value.type_of(&scope),
            UnaryOperation::Not { value, .. } => value.type_of(&scope),
        }
    }
}

impl TypeOf for Range {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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
        StaticType::build_range_from(type_sig, self.inclusive, scope).map(|value| e_static!(value))
    }
}

impl TypeOf for Product {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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
impl TypeOf for Addition {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_addition(&right_type, scope)
    }
}

impl TypeOf for Substraction {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_substraction(&right_type, scope)
    }
}

impl TypeOf for Shift {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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
impl TypeOf for BitwiseAnd {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_and(&right_type, scope)
    }
}
impl TypeOf for BitwiseXOR {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_xor(&right_type, scope)
    }
}
impl TypeOf for BitwiseOR {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_bitwise_or(&right_type, scope)
    }
}

impl TypeOf for Cast {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.cast(&right_type, scope)
    }
}

impl TypeOf for Comparaison {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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

impl TypeOf for Equation {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
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

impl TypeOf for LogicalAnd {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_and(&right_type, scope)
    }
}
impl TypeOf for LogicalOr {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope)?;
        let right_type = self.right.type_of(&scope)?;
        left_type.merge_logical_or(&right_type, scope)
    }
}
