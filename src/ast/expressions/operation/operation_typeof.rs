use crate::{
    ast::expressions::{
        data::{Data, Variable},
        Atomic, Expression,
    },
    e_static,
    semantic::{
        scope::{
            static_types::{self, ClosureType, FnType, StaticType},
            type_traits::OperandMerging,
        },
        EType, MergeType, Resolve, SemanticError, TypeOf,
    },
};

use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, FieldAccess, FnCall,
    ListAccess, LogicalAnd, LogicalOr, Product, Range, Shift, Substraction, TupleAccess,
    UnaryOperation,
};
use crate::semantic::scope::scope::ScopeManager;

impl TypeOf for UnaryOperation {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            UnaryOperation::Minus { value, .. } => value.type_of(&scope_manager, scope_id),
            UnaryOperation::Not { value, .. } => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl TypeOf for TupleAccess {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
impl TypeOf for ListAccess {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
impl TypeOf for FieldAccess {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for FnCall {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        if self.is_dynamic_fn.is_some() || self.platform.is_some() {
            return self
                .metadata
                .signature()
                .ok_or(SemanticError::NotResolvedYet);
        }

        let fn_var_type = self.fn_var.type_of(&scope_manager, scope_id)?;

        let return_type = match fn_var_type {
            EType::Static(StaticType::Closure(ClosureType { ref ret, .. })) => ret.as_ref().clone(),
            EType::Static(StaticType::StaticFn(FnType { ref ret, .. })) => ret.as_ref().clone(),
            _ => return Err(SemanticError::ExpectedCallable),
        };

        Ok(return_type)
    }
}

impl TypeOf for Range {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let type_sig = match self.lower.type_of(scope_manager, scope_id)?.merge(
            &self.upper.type_of(scope_manager, scope_id)?,
            scope_manager,
            scope_id,
        )? {
            EType::Static(value) => match value {
                StaticType::Primitive(static_types::PrimitiveType::Number(value)) => value.clone(),
                _ => return Err(SemanticError::IncompatibleTypes),
            },
            EType::User { .. } => return Err(SemanticError::IncompatibleTypes),
        };
        Ok(EType::Static(static_types::StaticType::Range(
            static_types::RangeType {
                num: type_sig,
                inclusive: self.inclusive,
            },
        )))
    }
}

impl TypeOf for Product {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Product::Mult {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(left_type)
            }
            Product::Div {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(left_type)
            }
            Product::Mod {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(left_type)
            }
        }
    }
}
impl TypeOf for Addition {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(left_type)
    }
}

impl TypeOf for Substraction {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(left_type)
    }
}

impl TypeOf for Shift {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Shift::Left {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(left_type)
            }
            Shift::Right {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(left_type)
            }
        }
    }
}
impl TypeOf for BitwiseAnd {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(left_type)
    }
}
impl TypeOf for BitwiseXOR {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(left_type)
    }
}
impl TypeOf for BitwiseOR {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(left_type)
    }
}

impl TypeOf for Cast {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(right_type)
    }
}

impl TypeOf for Comparaison {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Comparaison::Less {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
            Comparaison::LessEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
            Comparaison::Greater {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
            Comparaison::GreaterEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
        }
    }
}

impl TypeOf for Equation {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Equation::Equal {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
            Equation::NotEqual {
                left,
                right,
                metadata: _,
            } => {
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Bool,
                )))
            }
        }
    }
}

impl TypeOf for LogicalAnd {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(EType::Static(StaticType::Primitive(
            static_types::PrimitiveType::Bool,
        )))
    }
}
impl TypeOf for LogicalOr {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;
        Ok(EType::Static(StaticType::Primitive(
            static_types::PrimitiveType::Bool,
        )))
    }
}
