use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, FieldAccess, FnCall,
    ListAccess, LogicalAnd, LogicalOr, Product, Range, Shift, Substraction, TupleAccess,
    UnaryOperation,
};

use crate::ast::expressions::data::{Data, Primitive, Variable};
use crate::ast::expressions::{Atomic, Expression};
use crate::p_num;
use crate::semantic::scope::static_types::{
    ClosureType, FnType, NumberType, PrimitiveType, RangeType, SliceType, StaticType, StrSliceType,
    TupleType, VecType,
};
use crate::semantic::scope::user_type_impl::{Struct, UserType};
use crate::semantic::{CompatibleWith, EType, Resolve, SemanticError, TypeOf};
use crate::semantic::{Info, ResolveFromStruct, SizeOf};
use crate::vm::platform::Lib;
use crate::vm::vm::DynamicFnResolver;

impl Resolve for UnaryOperation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
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
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
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

impl Resolve for TupleAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, context, extra)?;

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
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<G>(scope_manager, scope_id, struct_id)?;
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

impl Resolve for ListAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, context, extra)?;
        let var_type = self.var.type_of(&scope_manager, scope_id)?;
        let _ = self
            .index
            .resolve::<G>(scope_manager, scope_id, &Some(p_num!(U64)), extra)?;
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
            EType::Static(StaticType::StrSlice(StrSliceType { size })) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Char))),
                };
                Ok(())
            }
            EType::Static(StaticType::String(_)) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Char))),
                };
                Ok(())
            }
            _ => Err(SemanticError::ExpectedIndexable),
        }
    }
}

impl ResolveFromStruct for ListAccess {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<G>(scope_manager, scope_id, struct_id)?;
        let var_type = self.var.type_of(&scope_manager, scope_id)?;
        let _ = self
            .index
            .resolve::<G>(scope_manager, scope_id, &Some(p_num!(U64)), &mut None)?;
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
            EType::Static(StaticType::StrSlice(StrSliceType { size })) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Char))),
                };
                Ok(())
            }
            EType::Static(StaticType::String(_)) => {
                self.metadata.info = Info::Resolved {
                    context: Some(var_type.clone()),
                    signature: Some(EType::Static(StaticType::Primitive(PrimitiveType::Char))),
                };
                Ok(())
            }
            _ => Err(SemanticError::ExpectedIndexable),
        }
    }
}

impl Resolve for FieldAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, context, extra)?;
        let EType::User { id, .. } = self.var.type_of(&scope_manager, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };
        let UserType::Struct(Struct { .. }) = scope_manager.find_type_by_id(id, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };

        self.field
            .resolve_from_struct::<G>(scope_manager, scope_id, id)?;

        Ok(())
    }
}

impl ResolveFromStruct for FieldAccess {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .var
            .resolve_from_struct::<G>(scope_manager, scope_id, struct_id)?;

        let EType::User { id, .. } = self.var.type_of(&scope_manager, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };
        let UserType::Struct(Struct { .. }) = scope_manager.find_type_by_id(id, scope_id)? else {
            return Err(SemanticError::ExpectedStruct);
        };

        self.field
            .resolve_from_struct::<G>(scope_manager, scope_id, id)?;

        Ok(())
    }
}

impl Resolve for FnCall {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self.fn_var.as_mut() {
            Expression::Atomic(Atomic::Data(Data::Variable(Variable { ref name, .. }))) => {
                let found = scope_manager.find_var_by_name(name, scope_id);
                if found.is_err() || self.lib.is_some() {
                    if let Some(mut dynamic_fn) = G::is_dynamic_fn(&self.lib, name) {
                        self.is_dynamic_fn = Some(name.as_str().to_string());
                        let return_type =
                            dynamic_fn.resolve::<G>(scope_manager, scope_id, &mut self.params)?;
                        self.metadata.info = crate::semantic::Info::Resolved {
                            context: context.clone(),
                            signature: Some(return_type),
                        };
                        return Ok(());
                    }
                    if let Some(mut api) = Lib::from(&self.lib, name) {
                        let _ =
                            api.resolve::<G>(scope_manager, scope_id, context, &mut self.params)?;

                        self.platform = Some(api);

                        self.metadata.info = crate::semantic::Info::Resolved {
                            context: context.clone(),
                            signature: Some(self.type_of(scope_manager, scope_id)?),
                        };
                        return Ok(());
                    }
                }
            }
            _ => {}
        }

        let _ = self
            .fn_var
            .resolve::<G>(scope_manager, scope_id, context, extra)?;

        let fn_var_type = self.fn_var.type_of(&scope_manager, scope_id)?;

        let params_type = match fn_var_type {
            EType::Static(StaticType::Closure(ClosureType { ref params, .. })) => params.clone(),
            EType::Static(StaticType::StaticFn(FnType { ref params, .. })) => params.clone(),
            _ => return Err(SemanticError::ExpectedCallable),
        };

        if self.params.len() != params_type.len() {
            return Err(SemanticError::IncorrectArguments);
        }
        for (expr, param_type) in self.params.iter_mut().zip(params_type) {
            let _ = expr.resolve::<G>(
                scope_manager,
                scope_id,
                &Some(param_type.clone()),
                &mut None,
            )?;
            let _ = expr.type_of(scope_manager, scope_id)?.compatible_with(
                &param_type,
                scope_manager,
                scope_id,
            )?;
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl ResolveFromStruct for FnCall {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let _ = self
            .fn_var
            .resolve_from_struct::<G>(scope_manager, scope_id, struct_id)?;

        todo!();
        Ok(())
    }
}

impl Resolve for Range {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_context = match context {
            Some(context) => match context {
                EType::Static(context) => match context {
                    StaticType::Range(RangeType { num, .. }) => Some(EType::Static(
                        StaticType::Primitive(PrimitiveType::Number(num.clone())).into(),
                    )),
                    _ => None,
                },
                _ => None,
            },
            None => None,
        };

        match (self.lower.as_mut(), self.upper.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let upper_type = self.upper.type_of(&scope_manager, scope_id)?;
                let _ = self.lower.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(upper_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let lower_type = self.lower.type_of(&scope_manager, scope_id)?;
                let _ = self.upper.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(lower_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .lower
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .upper
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let _ = self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

fn is_number(ctype: &EType) -> bool {
    match ctype {
        EType::Static(StaticType::Primitive(PrimitiveType::Number(_))) => true,
        _ => false,
    }
}

impl Resolve for Product {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                let _ = left.resolve::<G>(scope_manager, scope_id, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }

        Ok(())
    }
}
impl Resolve for Addition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }

        Ok(())
    }
}

impl Resolve for Substraction {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Resolve for Shift {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                let _ = left.resolve::<G>(scope_manager, scope_id, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }

        Ok(())
    }
}
impl Resolve for BitwiseAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}
impl Resolve for BitwiseXOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }
        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}
impl Resolve for BitwiseOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = self.left.type_of(&scope_manager, scope_id)?;
        let right_type = self.right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Resolve for Cast {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
        let _ = self
            .right
            .resolve::<G>(scope_manager, scope_id, &(), &mut ())?;

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

impl Resolve for Comparaison {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                let _ = left.resolve::<G>(scope_manager, scope_id, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if is_number(&left_type) && is_number(&right_type) && left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Resolve for Equation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                let _ = left.resolve::<G>(scope_manager, scope_id, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = left.type_of(&scope_manager, scope_id)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = right.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&scope_manager, scope_id)?;
        let right_type = right.type_of(&scope_manager, scope_id)?;

        if left_type == right_type {
            metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(left_type),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
        Ok(())
    }
}

impl Resolve for LogicalAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

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
impl Resolve for LogicalOr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let right_type = self.right.type_of(&scope_manager, scope_id)?;
                let _ = self.left.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(right_type),
                    &mut None,
                )?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let left_type = self.left.type_of(&scope_manager, scope_id)?;
                let _ = self.right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(left_type),
                    &mut None,
                )?;
            }
            _ => {
                let _ = self
                    .left
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
                let _ = self
                    .right
                    .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
            }
        }

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

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expressions::{operation::operation_parse::TryParseOperation, Expression},
            TryParse,
        },
        p_num,
        semantic::scope::static_types::{FnType, StaticType},
    };

    use super::*;

    #[test]
    fn valid_high_ord_math() {
        let mut expr = Product::parse("10 * 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10.0 * 10.0".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * (10+10)".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("(10+10) * 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Product::parse("'a' * 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 - 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + (10*10)".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + 10*10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("(10 * 10) + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Addition::parse("'a' + 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Addition::parse("10.0 + 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Shift::parse("10 << 1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Shift::parse("'a' >> 1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseOR::parse("10 | 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseXOR::parse("10 ^ 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = BitwiseOR::parse("'a' | 10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = BitwiseXOR::parse("10 ^ 'a'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' > 'b'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' != 'b'".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true or false".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true and 2 > 3".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());

        let mut expr = Expression::parse("1 or true".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
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
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("! ( true and false )".into())
            .unwrap()
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1.0".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("- ( 10 + 10 )".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_unary() {
        let mut expr = UnaryOperation::parse("!10".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_err());

        let mut expr = UnaryOperation::parse("- true".into()).unwrap().1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_call() {
        let mut expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_var(
                "f",
                EType::Static(
                    StaticType::StaticFn(FnType {
                        params: vec![p_num!(I64), p_num!(I64)],
                        ret: Box::new(p_num!(I64)),
                        scope_params_size: 24,
                    })
                    .into(),
                ),
                None,
            )
            .unwrap();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let ret_type = expr.type_of(&scope_manager, None).unwrap();
        assert_eq!(ret_type, p_num!(I64))
    }

    #[test]
    fn robustness_call() {
        let mut expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("f", p_num!(I64), None).unwrap();
        let res = expr.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_call_lib() {
        let mut expr = FnCall::parse("io::print(true)".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_var("print", p_num!(I64), None)
            .unwrap();
        let _ = expr
            .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut None)
            .expect("Resolution should have succeeded");
    }

    #[test]
    fn robustness_call_lib() {
        let mut expr = FnCall::parse("print(true)".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_var("print", p_num!(I64), None)
            .unwrap();
        let _ = expr
            .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut None)
            .expect_err("Resolution should have failed");
    }
}
