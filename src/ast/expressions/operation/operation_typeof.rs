use crate::semantic::{
    scope::static_types::{self, StaticType},
    EType, Resolve, SemanticError, TypeOf,
};

use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, ExprCall,
    FieldAccess, ListAccess, LogicalAnd, LogicalOr, Product, Shift, Substraction, TupleAccess,
    UnaryOperation,
};

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
impl TypeOf for ExprCall {
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
        Ok(EType::Static(StaticType::Primitive(
            static_types::PrimitiveType::Bool,
        )))
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
        Ok(EType::Static(StaticType::Primitive(
            static_types::PrimitiveType::Bool,
        )))
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
        Ok(EType::Static(StaticType::Primitive(
            static_types::PrimitiveType::Bool,
        )))
    }
}
