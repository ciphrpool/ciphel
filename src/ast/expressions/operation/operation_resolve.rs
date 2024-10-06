use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, ExprCall,
    FieldAccess, ListAccess, LogicalAnd, LogicalOr, Product, Shift, Substraction, TupleAccess,
    UnaryOperation,
};

use crate::ast::expressions::{Atomic, Expression};
use crate::p_num;
use crate::semantic::scope::static_types::{
    ClosureType, FunctionType, LambdaType, NumberType, PrimitiveType, SliceType, StaticType,
    TupleType, VecType,
};
use crate::semantic::scope::user_types::{Struct, UserType};
use crate::semantic::{Desugar, EType, Resolve, ResolveCore, ResolveNumber, SemanticError, TypeOf};
use crate::semantic::{Info, ResolveFromStruct, SizeOf};

impl Resolve for UnaryOperation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            UnaryOperation::Minus { value, metadata } => {
                if value.is_unresolved_number() {
                    value.resolve_number(NumberType::I64)?;
                }

                let _ = value.resolve::<E>(scope_manager, scope_id, context, &mut None)?;

                let value_type = value.type_of(&scope_manager, scope_id)?;

                match value_type {
                    EType::Static(value) => match value {
                        StaticType::Primitive(value) => match value {
                            PrimitiveType::Number(
                                NumberType::I128
                                | NumberType::I64
                                | NumberType::I32
                                | NumberType::I16
                                | NumberType::I8
                                | NumberType::F64,
                            ) => {}
                            _ => return Err(SemanticError::IncompatibleOperation),
                        },
                        _ => return Err(SemanticError::IncompatibleOperation),
                    },
                    EType::User { .. } => return Err(SemanticError::IncompatibleOperation),
                }

                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(value.type_of(&scope_manager, scope_id)?),
                };

                Ok(())
            }
            UnaryOperation::Not { value, metadata } => {
                let _ = value.resolve::<E>(scope_manager, scope_id, context, &mut None)?;
                let value_type = value.type_of(&scope_manager, scope_id)?;
                let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) = value_type else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
                };

                Ok(())
            }
        }
    }
}

impl Desugar<Atomic> for UnaryOperation {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Atomic>, SemanticError> {
        match self {
            UnaryOperation::Minus { value, .. } => {
                let Some(output) = value.desugar::<E>(scope_manager, scope_id)? else {
                    return Ok(None);
                };
                *value = output.into();
            }
            UnaryOperation::Not { value, .. } => {
                let Some(output) = value.desugar::<E>(scope_manager, scope_id)? else {
                    return Ok(None);
                };
                *value = output.into();
            }
        };

        Ok(None)
    }
}

impl ResolveNumber for UnaryOperation {
    fn is_unresolved_number(&self) -> bool {
        match self {
            UnaryOperation::Minus { value, metadata } => value.is_unresolved_number(),
            UnaryOperation::Not { value, metadata } => value.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            UnaryOperation::Minus { value, metadata } => value.resolve_number(to),
            UnaryOperation::Not { value, metadata } => value.resolve_number(to),
        }
    }
}

impl Resolve for TupleAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .var
            .resolve::<E>(scope_manager, scope_id, context, extra)?;

        let var_type = self.var.type_of(&scope_manager, scope_id)?;

        let EType::Static(StaticType::Tuple(TupleType(fields))) = var_type.clone() else {
            return Err(SemanticError::ExpectedIndexable);
        };

        if self.index >= fields.len() {
            return Err(SemanticError::InvalidPattern);
        }
        let mut idx = 0;
        let mut offset = 0;
        while idx < self.index {
            offset += fields[idx].size_of();
            idx += 1;
        }

        let item_type = fields
            .get(self.index)
            .ok_or(SemanticError::ExpectedIndexable)?;

        self.offset.insert(offset);

        self.metadata.info = Info::Resolved {
            context: Some(var_type),
            signature: Some(item_type.clone()),
        };
        Ok(())
    }
}

impl ResolveFromStruct for TupleAccess {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<E>(scope_manager, scope_id, struct_id)?;
        let var_type = self.var.type_of(&scope_manager, scope_id)?;

        let EType::Static(StaticType::Tuple(TupleType(fields))) = var_type.clone() else {
            return Err(SemanticError::ExpectedIndexable);
        };

        if self.index >= fields.len() {
            return Err(SemanticError::InvalidPattern);
        }
        let mut idx = 0;
        let mut offset = 0;
        while idx < self.index {
            offset += fields[idx].size_of();
            idx += 1;
        }

        let item_type = fields
            .get(self.index)
            .ok_or(SemanticError::ExpectedIndexable)?;

        self.offset.insert(offset);

        self.metadata.info = Info::Resolved {
            context: Some(var_type),
            signature: Some(item_type.clone()),
        };
        Ok(())
    }
}

impl Desugar<Expression> for TupleAccess {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.var.desugar::<E>(scope_manager, scope_id)? {
            self.var = output.into();
        }
        Ok(None)
    }
}

impl Resolve for ListAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .var
            .resolve::<E>(scope_manager, scope_id, context, extra)?;
        let var_type = self.var.type_of(&scope_manager, scope_id)?;
        let _ = self
            .index
            .resolve::<E>(scope_manager, scope_id, &Some(p_num!(U64)), extra)?;
        match &var_type {
            EType::Static(StaticType::Slice(SliceType { size, item_type })) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(item_type.as_ref().clone()),
                };
                Ok(())
            }
            EType::Static(StaticType::Vec(VecType(item_type))) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(item_type.as_ref().clone()),
                };
                Ok(())
            }
            _ => Err(SemanticError::ExpectedIndexable),
        }
    }
}

impl ResolveFromStruct for ListAccess {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<E>(scope_manager, scope_id, struct_id)?;
        let var_type = self.var.type_of(&scope_manager, scope_id)?;
        let _ = self
            .index
            .resolve::<E>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
        match &var_type {
            EType::Static(StaticType::Slice(SliceType { size, item_type })) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(item_type.as_ref().clone()),
                };
                Ok(())
            }
            EType::Static(StaticType::Vec(VecType(item_type))) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(item_type.as_ref().clone()),
                };
                Ok(())
            }
            _ => Err(SemanticError::ExpectedIndexable),
        }
    }
}

impl Desugar<Expression> for ListAccess {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.var.desugar::<E>(scope_manager, scope_id)? {
            self.var = output.into();
        }
        Ok(None)
    }
}

impl Resolve for FieldAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .var
            .resolve::<E>(scope_manager, scope_id, context, extra)?;

        let EType::User { id, .. } = self.var.type_of(&scope_manager, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };
        let UserType::Struct(Struct { .. }) = scope_manager.find_type_by_id(id, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };

        self.field
            .resolve_from_struct::<E>(scope_manager, scope_id, id)?;

        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(self.field.type_of(scope_manager, scope_id)?),
        };

        Ok(())
    }
}

impl ResolveFromStruct for FieldAccess {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<E>(scope_manager, scope_id, struct_id)?;

        let EType::User { id, size } = self.var.type_of(&scope_manager, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };
        let UserType::Struct(Struct { .. }) = scope_manager.find_type_by_id(id, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };

        self.field
            .resolve_from_struct::<E>(scope_manager, scope_id, id)?;

        self.metadata.info = Info::Resolved {
            context: Some(EType::User { id, size }),
            signature: Some(self.field.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl Desugar<Expression> for FieldAccess {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.var.desugar::<E>(scope_manager, scope_id)? {
            self.var = output.into();
        }
        Ok(None)
    }
}

impl Resolve for ExprCall {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .var
            .resolve::<E>(scope_manager, scope_id, &None, extra)?;

        let var_type = self.var.type_of(scope_manager, scope_id)?;
        let (params, return_type) = match &var_type {
            EType::Static(StaticType::Closure(ClosureType { params, ret, .. })) => (params, ret),
            EType::Static(StaticType::Lambda(LambdaType { params, ret, .. })) => (params, ret),
            EType::Static(StaticType::Function(FunctionType { params, ret, .. })) => (params, ret),
            _ => return Err(SemanticError::ExpectedCallable),
        };

        if params.len() != self.args.args.len() {
            return Err(SemanticError::IncorrectArguments);
        }

        let mut params_size = 0;
        for (arg, param) in self.args.args.iter_mut().zip(params) {
            let _ = arg.resolve::<E>(scope_manager, scope_id, &Some(param.clone()), &mut None)?;
            params_size += param.size_of();
        }
        let _ = self.args.size.insert(params_size);

        if context.is_some() && *context.as_ref().unwrap() != **return_type {
            return Err(SemanticError::IncompatibleTypes);
        }
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(return_type.as_ref().clone()),
        };
        Ok(())
    }
}

impl ResolveFromStruct for ExprCall {
    fn resolve_from_struct<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<E>(scope_manager, scope_id, struct_id)?;

        let var_type = self.var.type_of(scope_manager, scope_id)?;
        let (params, return_type) = match &var_type {
            EType::Static(StaticType::Closure(ClosureType { params, ret, .. })) => (params, ret),
            EType::Static(StaticType::Lambda(LambdaType { params, ret, .. })) => (params, ret),
            EType::Static(StaticType::Function(FunctionType { params, ret, .. })) => (params, ret),
            _ => return Err(SemanticError::ExpectedCallable),
        };

        if params.len() != self.args.args.len() {
            return Err(SemanticError::IncorrectArguments);
        }

        let mut params_size = 0;
        for (arg, param) in self.args.args.iter_mut().zip(params) {
            let _ = arg.resolve::<E>(scope_manager, scope_id, &Some(param.clone()), &mut None)?;
            params_size += param.size_of();
        }
        let _ = self.args.size.insert(params_size);

        self.metadata.info = Info::Resolved {
            context: None,
            signature: Some(return_type.as_ref().clone()),
        };
        Ok(())
    }
}

impl Desugar<Expression> for ExprCall {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.var.desugar::<E>(scope_manager, scope_id)? {
            self.var = output.into();
        }
        Ok(None)
    }
}

fn is_number(ctype: &EType) -> bool {
    match ctype {
        EType::Static(StaticType::Primitive(PrimitiveType::Number(_))) => true,
        _ => false,
    }
}

fn resolve_logical_binary_operation<E: crate::vm::external::Engine>(
    left: &mut Expression,
    right: &mut Expression,
    scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
    scope_id: Option<u128>,
) -> Result<(), SemanticError> {
    let left_is_unresolved = left.is_unresolved_number();
    let right_is_unresolved = right.is_unresolved_number();

    if left_is_unresolved && right_is_unresolved {
        let _ = left.resolve_number(NumberType::I64);
        let _ = right.resolve_number(NumberType::I64);
        let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
    } else if left_is_unresolved && !right_is_unresolved {
        let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        if let EType::Static(StaticType::Primitive(PrimitiveType::Number(number_type))) =
            right.type_of(scope_manager, scope_id)?
        {
            let _ = left.resolve_number(number_type);
        }
        let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
    } else if right_is_unresolved && !left_is_unresolved {
        let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        if let EType::Static(StaticType::Primitive(PrimitiveType::Number(number_type))) =
            left.type_of(scope_manager, scope_id)?
        {
            let _ = right.resolve_number(number_type);
        }
        let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
    } else {
        let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
    }
    Ok(())
}
fn resolve_binary_numerical_operation<E: crate::vm::external::Engine>(
    left: &mut Expression,
    right: &mut Expression,
    scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
    scope_id: Option<u128>,
    context: &Option<EType>,
) -> Result<EType, SemanticError> {
    match context {
        Some(context_type) => {
            let EType::Static(StaticType::Primitive(PrimitiveType::Number(number_type))) =
                context_type
            else {
                return Err(SemanticError::IncompatibleOperands);
            };
            let _ = left.resolve_number(number_type.clone());
            let _ = right.resolve_number(number_type.clone());

            let _ = right.resolve::<E>(scope_manager, scope_id, context, &mut None)?;
            let _ = left.resolve::<E>(scope_manager, scope_id, context, &mut None)?;
        }
        None => {
            let left_is_unresolved = left.is_unresolved_number();
            let right_is_unresolved = right.is_unresolved_number();

            if left_is_unresolved && right_is_unresolved {
                let _ = left.resolve_number(NumberType::I64);
                let _ = right.resolve_number(NumberType::I64);
                let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            } else if left_is_unresolved && !right_is_unresolved {
                let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let EType::Static(StaticType::Primitive(PrimitiveType::Number(number_type))) =
                    right.type_of(scope_manager, scope_id)?
                else {
                    return Err(SemanticError::IncompatibleOperands);
                };
                let _ = left.resolve_number(number_type);
                let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            } else if right_is_unresolved && !left_is_unresolved {
                let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let EType::Static(StaticType::Primitive(PrimitiveType::Number(number_type))) =
                    left.type_of(scope_manager, scope_id)?
                else {
                    return Err(SemanticError::IncompatibleOperands);
                };
                let _ = right.resolve_number(number_type);
                let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            } else {
                let _ = right.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let _ = left.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            }
        }
    }

    let left_type = left.type_of(&scope_manager, scope_id)?;
    let right_type = right.type_of(&scope_manager, scope_id)?;

    if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
        return Ok(left_type);
    } else {
        return Err(SemanticError::IncompatibleTypes);
    }
}

impl ResolveNumber for Product {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Product::Div {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Product::Mod {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Product::Div {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Product::Mod {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
        }
    }
}
impl Resolve for Product {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Product::Div {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Product::Mod {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        let operation_type =
            resolve_binary_numerical_operation::<E>(left, right, scope_manager, scope_id, context)?;
        metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };

        Ok(())
    }
}

impl Desugar<Expression> for Product {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        match self {
            Product::Mult { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Product::Div { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Product::Mod { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
        }
        Ok(None)
    }
}

impl ResolveNumber for Addition {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for Addition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let operation_type = resolve_binary_numerical_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
            context,
        )?;
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };

        Ok(())
    }
}

impl Desugar<Expression> for Addition {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}

impl ResolveNumber for Substraction {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for Substraction {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let operation_type = resolve_binary_numerical_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
            context,
        )?;
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };
        Ok(())
    }
}

impl Desugar<Expression> for Substraction {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}

impl ResolveNumber for Shift {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Shift::Left {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Shift::Right {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Shift::Left {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Shift::Right {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
        }
    }
}
impl Resolve for Shift {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Shift::Left {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Shift::Right {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        let operation_type =
            resolve_binary_numerical_operation::<E>(left, right, scope_manager, scope_id, context)?;
        metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };
        Ok(())
    }
}

impl Desugar<Expression> for Shift {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        match self {
            Shift::Left { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Shift::Right { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
        }
        Ok(None)
    }
}

impl ResolveNumber for BitwiseAnd {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for BitwiseAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let operation_type = resolve_binary_numerical_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
            context,
        )?;
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };
        Ok(())
    }
}

impl Desugar<Expression> for BitwiseAnd {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}
impl ResolveNumber for BitwiseXOR {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}

impl Desugar<Expression> for BitwiseXOR {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}
impl Resolve for BitwiseXOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let operation_type = resolve_binary_numerical_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
            context,
        )?;
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };
        Ok(())
    }
}

impl ResolveNumber for BitwiseOR {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for BitwiseOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let operation_type = resolve_binary_numerical_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
            context,
        )?;
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(operation_type),
        };
        Ok(())
    }
}

impl Desugar<Expression> for BitwiseOR {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}
impl ResolveNumber for Cast {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        Ok(())
    }
}

impl Resolve for Cast {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .left
            .resolve::<E>(scope_manager, scope_id, context, &mut None)?;
        let _ = self
            .right
            .resolve::<E>(scope_manager, scope_id, &(), &mut ())?;

        if let Some(l_metadata) = self.left.metadata_mut() {
            let signature = l_metadata.signature();
            let new_context = self.right.type_of(&scope_manager, scope_id)?;

            l_metadata.info = Info::Resolved {
                context: Some(new_context),
                signature,
            };
        }
        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl Desugar<Expression> for Cast {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        Ok(None)
    }
}

impl ResolveNumber for Comparaison {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
        }
    }
}

impl Resolve for Comparaison {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        resolve_logical_binary_operation::<E>(left, right, scope_manager, scope_id)?;
        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Desugar<Expression> for Comparaison {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        match self {
            Comparaison::Greater { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Comparaison::GreaterEqual { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Comparaison::Less { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Comparaison::LessEqual { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
        }
        Ok(None)
    }
}

impl ResolveNumber for Equation {
    fn is_unresolved_number(&self) -> bool {
        match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => left.is_unresolved_number() || right.is_unresolved_number(),
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => {
                left.resolve_number(to)?;
                right.resolve_number(to)?;
                Ok(())
            }
        }
    }
}
impl Resolve for Equation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };
        resolve_logical_binary_operation::<E>(left, right, scope_manager, scope_id)?;
        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Desugar<Expression> for Equation {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        match self {
            Equation::Equal { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
            Equation::NotEqual { left, right, .. } => {
                if let Some(output) = left.desugar::<E>(scope_manager, scope_id)? {
                    *left = output.into()
                }
                if let Some(output) = right.desugar::<E>(scope_manager, scope_id)? {
                    *right = output.into()
                }
            }
        }
        Ok(None)
    }
}

impl ResolveNumber for LogicalAnd {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for LogicalAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        resolve_logical_binary_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
        )?;
        let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) =
            self.left.type_of(&scope_manager, scope_id)?
        else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) =
            self.right.type_of(&scope_manager, scope_id)?
        else {
            return Err(SemanticError::IncompatibleTypes);
        };
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
        };
        Ok(())
    }
}

impl Desugar<Expression> for LogicalAnd {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}
impl ResolveNumber for LogicalOr {
    fn is_unresolved_number(&self) -> bool {
        self.left.is_unresolved_number() || self.right.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.left.resolve_number(to)?;
        self.right.resolve_number(to)?;
        Ok(())
    }
}
impl Resolve for LogicalOr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        resolve_logical_binary_operation::<E>(
            &mut self.left,
            &mut self.right,
            scope_manager,
            scope_id,
        )?;

        let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) =
            self.left.type_of(&scope_manager, scope_id)?
        else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) =
            self.right.type_of(&scope_manager, scope_id)?
        else {
            return Err(SemanticError::IncompatibleTypes);
        };
        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
        };
        Ok(())
    }
}
impl Desugar<Expression> for LogicalOr {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Expression>, SemanticError> {
        if let Some(output) = self.left.desugar::<E>(scope_manager, scope_id)? {
            *self.left = output.into()
        }
        if let Some(output) = self.right.desugar::<E>(scope_manager, scope_id)? {
            *self.right = output.into()
        }
        Ok(None)
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{operation::operation_parse::TryParseOperation, Expression},
            TryParse,
        },
        p_num,
    };

    use super::*;

    #[test]
    fn valid_high_ord_math() {
        let mut expr = Product::parse("10 * 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10.0 * 10.0".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * (10+10)".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("(10+10) * 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_high_ord_math() {
        let mut expr = Product::parse("10 * 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Product::parse("'a' * 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_low_ord_math() {
        let mut expr = Addition::parse("10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 - 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + (10*10)".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + 10*10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("(10 * 10) + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_low_ord_math() {
        let mut expr = Addition::parse("10 + 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Addition::parse("'a' + 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_shift() {
        let mut expr = Shift::parse("10 >> 1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Shift::parse("10 << 1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_shift() {
        let mut expr = Shift::parse("10 >> 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Shift::parse("'a' >> 1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_bitwise() {
        let mut expr = BitwiseAnd::parse("10 & 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseOR::parse("10 | 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseXOR::parse("10 ^ 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_bitwise() {
        let mut expr = BitwiseAnd::parse("10 & 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = BitwiseOR::parse("'a' | 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = BitwiseXOR::parse("10 ^ 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_cast() {
        let mut expr = Cast::parse("10 as f64".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope_manager, None).unwrap();
        assert_eq!(p_num!(F64), expr_type);

        let mut expr = Cast::parse("'a' as u64".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope_manager, None).unwrap();
        assert_eq!(p_num!(U64), expr_type);
    }

    #[test]
    fn valid_comparaison() {
        let mut expr = Expression::parse("10 < 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' > 'b'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_comparaison() {
        let mut expr = Expression::parse("10 > 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_equation() {
        let mut expr = Expression::parse("10 == 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' != 'b'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_equation() {
        let mut expr = Expression::parse("10 == 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_and_or() {
        let mut expr = Expression::parse("true and false".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true or false".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true and 2 > 3".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true and true and true".into())
            .unwrap()
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_and_or() {
        let mut expr = Expression::parse("true and 2".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Expression::parse("1 or true".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_unary() {
        let mut expr = UnaryOperation::parse("!true".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("! ( true and false )".into())
            .unwrap()
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1.0".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("- ( 10 + 10 )".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_unary() {
        let mut expr = UnaryOperation::parse("!10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());

        let mut expr = UnaryOperation::parse("- true".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    // #[test]
    // fn valid_call() {
    //     let mut expr = FnCall::parse("f(10,20+20)".into())
    //         .expect("Parsing should have succeeded")
    //         .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_var(
    //             "f",
    //             EType::Static(
    //                 StaticType::Function(FunctionType {
    //                     params: vec![p_num!(I64), p_num!(I64)],
    //                     ret: Box::new(p_num!(I64)),
    //                     scope_params_size: 24,
    //                 })
    //                 .into(),
    //             ),
    //             None,
    //         )
    //         .unwrap();
    //     let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
    //         &mut scope_manager,
    //         None,
    //         &None,
    //         &mut None,
    //     );
    //     assert!(res.is_ok(), "{:?}", res);

    //     let ret_type = expr.type_of(&scope_manager, None).unwrap();
    //     assert_eq!(ret_type, p_num!(I64))
    // }

    // #[test]
    // fn robustness_call() {
    //     let mut expr = FnCall::parse("f(10,20+20)".into())
    //         .expect("Parsing should have succeeded")
    //         .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager.register_var("f", p_num!(I64), None).unwrap();
    //     let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
    //         &mut scope_manager,
    //         None,
    //         &None,
    //         &mut None,
    //     );
    //     assert!(res.is_err());
    // }

    // #[test]
    // fn valid_call_lib() {
    //     let mut expr = FnCall::parse("io::print(true)".into())
    //         .expect("Parsing should have succeeded")
    //         .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_var("print", p_num!(I64), None)
    //         .unwrap();
    //     let _ = expr
    //         .resolve::<crate::vm::external::test::NoopEngine>(&mut scope_manager, None, &None, &mut None)
    //         .expect("Resolution should have succeeded");
    // }

    // #[test]
    // fn robustness_call_lib() {
    //     let mut expr = FnCall::parse("print(true)".into())
    //         .expect("Parsing should have succeeded")
    //         .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = scope_manager
    //         .register_var("print", p_num!(I64), None)
    //         .unwrap();
    //     let _ = expr
    //         .resolve::<crate::vm::external::test::NoopEngine>(&mut scope_manager, None, &None, &mut None)
    //         .expect_err("Resolution should have failed");
    // }
}
