use super::{
    Address, Closure, ClosureParam, Data, Enum, Map, Number, Primitive, PtrAccess, Slice, StrSlice,
    Struct, Tuple, Union, Variable, Vector,
};
use crate::ast::expressions::Atomic;
use crate::ast::statements::block::BlockCommonApi;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{
    AddrType, ClosureType, MapType, NumberType, PrimitiveType, SliceType, StrSliceType, TupleType,
    VecType,
};
use crate::semantic::scope::user_type_impl::UserType;
use crate::semantic::scope::ClosureState;
use crate::semantic::{
    scope::static_types::StaticType, CompatibleWith, EType, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{Info, MergeType, ResolveFromStruct, SizeOf};
use crate::vm::allocator::align;
use crate::{e_static, semantic};

impl Resolve for Data {
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
        match self {
            Data::Primitive(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Slice(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Vec(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Closure(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Tuple(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Address(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::PtrAccess(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Variable(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Struct(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Union(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::Enum(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
            Data::StrSlice(value) => value.resolve::<G>(scope_manager, scope_id, context, &mut ()),
        }
    }
}

impl ResolveFromStruct for Data {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        match self {
            Data::Variable(value) => {
                value.resolve_from_struct::<G>(scope_manager, scope_id, struct_id)
            }
            _ => Err(SemanticError::UnknownField),
        }
    }
}

impl Resolve for Variable {
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
        let var = scope_manager.find_var_by_name(&self.name, scope_id)?;
        self.state
            .insert(super::VariableState::Variable { id: var.id });
        let var_type = var.ctype.clone();

        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(var_type),
        };
        Ok(())
    }
}

impl ResolveFromStruct for Variable {
    fn resolve_from_struct<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        struct_id: u64,
    ) -> Result<(), SemanticError> {
        let UserType::Struct(struct_type @ semantic::scope::user_type_impl::Struct { .. }) =
            scope_manager.find_type_by_id(struct_id, scope_id)?
        else {
            return Err(SemanticError::ExpectedStruct);
        };

        let mut found = false;
        let mut offset = 0;

        for (field_name, field_type) in struct_type.fields.iter() {
            if *field_name == self.name {
                found = true;
                self.state.insert(super::VariableState::Field {
                    offset,
                    size: align(field_type.size_of()),
                });
                self.metadata.info = Info::Resolved {
                    context: Some(EType::User {
                        id: struct_id,
                        size: struct_type.size_of(),
                    }),
                    signature: Some(field_type.clone()),
                };
            }
            offset += align(field_type.size_of())
        }
        if !found {
            return Err(SemanticError::UnknownField);
        }

        Ok(())
    }
}

impl Resolve for Primitive {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(context_type) => {
                match context_type {
                    EType::Static(value) => {
                        if let Primitive::Number(n) = self {
                            if let Number::Unresolved(v) = n {
                                match value {
                                    StaticType::Primitive(PrimitiveType::Number(value)) => {
                                        match value {
                                            NumberType::U8 => {
                                                *n = Number::U8((*v).try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?);
                                            }
                                            NumberType::U16 => {
                                                *n =
                                                    Number::U16((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::U32 => {
                                                *n =
                                                    Number::U32((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::U64 => {
                                                *n =
                                                    Number::U64((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::U128 => {
                                                *n = Number::U128((*v).try_into().map_err(
                                                    |_| SemanticError::IncompatibleTypes,
                                                )?);
                                            }
                                            NumberType::I8 => {
                                                *n = Number::I8((*v).try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?);
                                            }
                                            NumberType::I16 => {
                                                *n =
                                                    Number::I16((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::I32 => {
                                                *n =
                                                    Number::I32((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::I64 => {
                                                *n =
                                                    Number::I64((*v).try_into().map_err(|_| {
                                                        SemanticError::IncompatibleTypes
                                                    })?);
                                            }
                                            NumberType::I128 => {
                                                *n = Number::I128((*v).try_into().map_err(
                                                    |_| SemanticError::IncompatibleTypes,
                                                )?);
                                            }
                                            NumberType::F64 => *n = Number::F64(*v as f64),
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    EType::User { .. } => {}
                }
                let _ = context_type.compatible_with(
                    &self.type_of(scope_manager, scope_id)?,
                    &scope_manager,
                    scope_id,
                )?;
                Ok(())
            }
            None => {
                if let Primitive::Number(n) = self {
                    if let Number::Unresolved(v) = n {
                        *n = Number::I64(*v)
                    }
                }
                Ok(())
            }
        }
    }
}
impl Resolve for Slice {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                EType::Static(StaticType::Slice(SliceType { size, item_type })) => {
                    let item_type = item_type.as_ref().clone();
                    let sitem_type = &Some(item_type.clone());
                    if self.value.len() > *size {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    for value in &mut self.value {
                        let _ =
                            value.resolve::<G>(scope_manager, scope_id, sitem_type, &mut None)?;
                        let _ = item_type.compatible_with(
                            &value.type_of(scope_manager, scope_id)?,
                            &scope_manager,
                            scope_id,
                        )?;
                    }

                    {
                        self.metadata.info = Info::Resolved {
                            context: context.clone(),
                            signature: Some(e_static!(StaticType::Slice(SliceType {
                                size: *size,
                                item_type: Box::new(item_type),
                            }))),
                        };
                    }
                    Ok(())
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                if self.value.is_empty() {
                    return Err(SemanticError::CantInferType(
                        "of an empty array".to_string(),
                    ));
                }
                let _ = self.value[0].resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let item_type = self.value[0].type_of(scope_manager, scope_id)?;

                for value in &mut self.value[1..] {
                    let _ = value.resolve::<G>(
                        scope_manager,
                        scope_id,
                        &Some(item_type.clone()),
                        &mut None,
                    )?;
                    let _ = item_type.compatible_with(
                        &value.type_of(scope_manager, scope_id)?,
                        &scope_manager,
                        scope_id,
                    )?;
                }

                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(e_static!(StaticType::Slice(SliceType {
                        size: self.value.len(),
                        item_type: Box::new(item_type),
                    }))),
                };

                Ok(())
            }
        }
    }
}

impl Resolve for StrSlice {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(context_type) => match context_type {
                EType::Static(value) => match value {
                    StaticType::StrSlice(StrSliceType { size }) => {
                        if *size < self.value.len() {
                            return Err(SemanticError::IncompatibleTypes);
                        } else if *size >= self.value.len() {
                            self.padding = *size - self.value.len();
                        }
                    }
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
                EType::User { .. } => return Err(SemanticError::IncompatibleTypes),
            },
            None => {}
        }
        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl Resolve for Vector {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                EType::Static(StaticType::Vec(VecType(item_type))) => {
                    let item_type = item_type.as_ref().clone();
                    let sitem_type = &Some(item_type.clone());

                    for value in &mut self.value {
                        let _ =
                            value.resolve::<G>(scope_manager, scope_id, sitem_type, &mut None)?;
                        let _ = item_type.compatible_with(
                            &value.type_of(scope_manager, scope_id)?,
                            &scope_manager,
                            scope_id,
                        )?;
                    }

                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Vec(VecType(Box::new(item_type),)))),
                    };

                    Ok(())
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                if self.value.is_empty() {
                    return Err(SemanticError::CantInferType(
                        "of an empty array".to_string(),
                    ));
                }
                let _ = self.value[0].resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let item_type = self.value[0].type_of(scope_manager, scope_id)?;

                for value in &mut self.value[1..] {
                    let _ = value.resolve::<G>(
                        scope_manager,
                        scope_id,
                        &Some(item_type.clone()),
                        &mut None,
                    )?;
                    let _ = item_type.compatible_with(
                        &value.type_of(scope_manager, scope_id)?,
                        &scope_manager,
                        scope_id,
                    )?;
                }

                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(e_static!(StaticType::Vec(VecType(Box::new(item_type),)))),
                };

                Ok(())
            }
        }
    }
}
impl Resolve for Tuple {
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
        match context {
            Some(rcontext) => match rcontext {
                EType::Static(StaticType::Tuple(TupleType(values_type))) => {
                    if values_type.len() != self.value.len() {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    for (value, value_type) in self.value.iter_mut().zip(values_type) {
                        let _ = value.resolve::<G>(
                            scope_manager,
                            scope_id,
                            &Some(value_type.clone()),
                            &mut None,
                        )?;
                        let _ = value_type.compatible_with(
                            &value.type_of(scope_manager, scope_id)?,
                            &scope_manager,
                            scope_id,
                        )?;
                    }

                    {
                        self.metadata.info = Info::Resolved {
                            context: context.clone(),
                            signature: Some(e_static!(StaticType::Tuple(TupleType(
                                values_type.clone()
                            )))),
                        };
                    }
                    Ok(())
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut values_type = Vec::new();
                for value in &mut self.value {
                    let _ = value.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                    values_type.push(value.type_of(&scope_manager, scope_id)?);
                }

                {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Tuple(TupleType(values_type)))),
                    };
                }
                Ok(())
            }
        }
    }
}

impl Resolve for Closure {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_scope = self.scope.init_from_parent(scope_manager, scope_id)?;

        match context {
            Some(context) => {
                let EType::Static(StaticType::Closure(ClosureType {
                    params,
                    ret,
                    closed,
                    scope_params_size,
                })) = context
                else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                for (param, expected_param) in self.params.iter_mut().zip(params) {
                    let _ = param.resolve::<G>(
                        scope_manager,
                        Some(inner_scope),
                        &Some(expected_param.clone()),
                        &mut (),
                    )?;
                }

                let _ = self.scope.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(ret.as_ref().clone()),
                    &mut (),
                )?;
            }
            None => {
                for param in self.params.iter_mut() {
                    let _ = param.resolve::<G>(scope_manager, Some(inner_scope), &None, &mut ())?;
                }

                let _ = self
                    .scope
                    .resolve::<G>(scope_manager, scope_id, &None, &mut ())?;
            }
        }

        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(&scope_manager, scope_id)?),
        };

        Ok(())
    }
}

impl Resolve for ClosureParam {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ClosureParam::Full(var) => {
                var.resolve::<G>(scope_manager, scope_id, &mut (), &mut ());
                let var_type = var.type_of(scope_manager, scope_id)?;
                let _ = scope_manager.register_parameter(&var.name, var_type.clone(), scope_id)?;

                if let Some(context) = context {
                    let _ = context.compatible_with(&var_type, scope_manager, scope_id)?;
                }
                Ok(())
            }
            ClosureParam::Minimal(value) => match context {
                Some(context) => {
                    let _ = scope_manager.register_parameter(&value, context.clone(), scope_id)?;
                    Ok(())
                }
                None => Err(SemanticError::CantInferType(format!(
                    "of the parameter {}",
                    value
                ))),
            },
        }
    }
}
impl Resolve for Address {
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
        match self.value.as_ref() {
            Atomic::Data(_) => {}
            Atomic::UnaryOperation(_) => return Err(SemanticError::IncompatibleTypes),
            Atomic::Paren(_) => {}
            Atomic::ExprFlow(_) => return Err(SemanticError::IncompatibleTypes),
        }
        let _ = self
            .value
            .resolve::<G>(scope_manager, scope_id, context, &mut None)?;
        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}
impl Resolve for PtrAccess {
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
            .value
            .resolve::<G>(scope_manager, scope_id, context, &mut None)?;

        let address_type = self.value.type_of(&scope_manager, scope_id)?;

        match &address_type {
            EType::Static(value) => match value {
                StaticType::Address(AddrType(sub)) => {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(sub.as_ref().clone()),
                    };
                }
                _ => {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Any)),
                    };
                }
            },
            _ => {
                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(e_static!(StaticType::Any)),
                };
            }
        }
        Ok(())
    }
}

impl Resolve for Struct {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let UserType::Struct(semantic::scope::user_type_impl::Struct { fields, .. }) =
            scope_manager.find_type_by_name(&self.id, scope_id)?.clone()
        else {
            return Err(SemanticError::ExpectedStruct);
        };

        if fields.len() != self.fields.len() {
            return Err(SemanticError::IncorrectStruct(self.id.to_string()));
        }

        for (ref field_name, expr) in &mut self.fields {
            let Some((_, field_type)) = fields.iter().find(|(n, _)| *n == *field_name) else {
                return Err(SemanticError::UnknownField);
            };

            let _ = expr.resolve::<G>(
                scope_manager,
                scope_id,
                &Some(field_type.clone()),
                &mut None,
            )?;
            let _ = field_type.compatible_with(
                &expr.type_of(scope_manager, scope_id)?,
                &scope_manager,
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
impl Resolve for Union {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let UserType::Union(semantic::scope::user_type_impl::Union { variants, .. }) =
            scope_manager
                .find_type_by_name(&self.typename, scope_id)?
                .clone()
        else {
            return Err(SemanticError::ExpectedStruct);
        };

        let Some((_, semantic::scope::user_type_impl::Struct { fields, .. })) =
            variants.iter().find(|(n, _)| *n == self.variant)
        else {
            return Err(SemanticError::UnknownField);
        };

        if fields.len() != self.fields.len() {
            return Err(SemanticError::IncorrectStruct(self.variant.to_string()));
        }

        for (ref field_name, expr) in &mut self.fields {
            let Some((_, field_type)) = fields.iter().find(|(n, _)| *n == *field_name) else {
                return Err(SemanticError::UnknownField);
            };

            let _ = expr.resolve::<G>(
                scope_manager,
                scope_id,
                &Some(field_type.clone()),
                &mut None,
            )?;
            let _ = field_type.compatible_with(
                &expr.type_of(scope_manager, scope_id)?,
                &scope_manager,
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

impl Resolve for Enum {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let UserType::Enum(semantic::scope::user_type_impl::Enum { values, .. }) = scope_manager
            .find_type_by_name(&self.typename, scope_id)?
            .clone()
        else {
            return Err(SemanticError::ExpectedStruct);
        };
        let Some(_) = values.iter().find(|n| **n == *self.value) else {
            return Err(SemanticError::UnknownField);
        };

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl Resolve for Map {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                EType::Static(StaticType::Map(MapType {
                    keys_type,
                    values_type,
                })) => {
                    let key_type = keys_type.as_ref().clone();
                    let skey_type = &Some(key_type);
                    let value_type = values_type.as_ref().clone();
                    let svalue_type = &Some(value_type);

                    for (key, value) in &mut self.fields {
                        let _ = key.resolve::<G>(scope_manager, scope_id, skey_type, &mut None)?;
                        let _ = keys_type.compatible_with(
                            &key.type_of(scope_manager, scope_id)?,
                            &scope_manager,
                            scope_id,
                        )?;
                        let _ =
                            value.resolve::<G>(scope_manager, scope_id, svalue_type, &mut None)?;
                        let _ = values_type.compatible_with(
                            &value.type_of(scope_manager, scope_id)?,
                            &scope_manager,
                            scope_id,
                        )?;
                    }

                    {
                        self.metadata.info = Info::Resolved {
                            context: context.clone(),
                            signature: Some(e_static!(StaticType::Map(MapType {
                                keys_type: keys_type.clone(),
                                values_type: values_type.clone(),
                            }))),
                        };
                    }
                    Ok(())
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut keys_type = e_static!(StaticType::Any);
                let mut values_type = e_static!(StaticType::Any);
                for (key, value) in &mut self.fields {
                    let _ = key.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                    keys_type = keys_type.merge(
                        &key.type_of(scope_manager, scope_id)?,
                        &scope_manager,
                        scope_id,
                    )?;
                    let _ = value.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                    values_type = values_type.merge(
                        &value.type_of(scope_manager, scope_id)?,
                        &scope_manager,
                        scope_id,
                    )?;
                }

                let skeys_type = Some(keys_type.clone());
                let svalues_type = Some(values_type.clone());
                for (key, value) in &mut self.fields {
                    let _ = key.resolve::<G>(scope_manager, scope_id, &skeys_type, &mut None)?;
                    let _ =
                        value.resolve::<G>(scope_manager, scope_id, &svalues_type, &mut None)?;
                }

                {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Map(MapType {
                            keys_type: Box::new(keys_type),
                            values_type: Box::new(values_type),
                        }))),
                    };
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{expressions::Expression, TryParse},
        p_num,
        semantic::scope::{
            scope::ScopeManager,
            static_types::{
                AddrType, MapType, PrimitiveType, SliceType, StaticType, StrSliceType, StringType,
                TupleType, VecType,
            },
            user_type_impl::{self, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_primitive() {
        let mut primitive = Primitive::parse("1".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = primitive.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res = primitive.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(p_num!(I64)),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_primitive() {
        let mut primitive = Primitive::parse("1".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = primitive.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_string() {
        let mut string = StrSlice::parse(r##""Hello World""##.into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = string.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res = string.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::StrSlice(StrSliceType {
                    size: "Hello World".len(),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_slice() {
        let mut slice = Slice::parse("[1,2]".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = slice.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res = slice.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Slice(SliceType {
                    size: 2,
                    item_type: Box::new(p_num!(I64)),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_string() {
        let mut string = StrSlice::parse(r##""Hello World""##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = string.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }
    #[test]
    fn robustness_slice() {
        let mut slice = Slice::parse("[1,2]".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = slice.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Slice(SliceType {
                    size: 2,
                    item_type: Box::new(EType::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_vector() {
        let mut vector = Vector::parse("vec[1,2,3]".into())
            .expect("Parsing should have succeeded")
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = vector.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res = vector.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Vec(VecType(Box::new(p_num!(I64)))).into(),
            )),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_vector() {
        let mut vector = Vector::parse("vec[1,2,3]".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = vector.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Vec(VecType(Box::new(EType::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                ))))
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_variable() {
        let mut variable = Variable::parse("x".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope_manager, None);
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_array() {
        let mut variable = Variable::parse("x[10]".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_variable_array_complex() {
        let mut variable = Variable::parse("x[10 + 10]".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_variable_array() {
        let mut variable = Expression::parse("x[\"Test\"]".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_var(
                "x",
                EType::Static(StaticType::Map(MapType {
                    keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                    values_type: Box::new(p_num!(I64)),
                })),
                None,
            )
            .unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_err());
    }
    #[test]
    fn valid_variable_tuple() {
        let mut variable = Expression::parse("x.0".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_var(
                "x",
                EType::Static(StaticType::Tuple(TupleType(vec![p_num!(I64), p_num!(I64)]))),
                None,
            )
            .unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope_manager, None);
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_struct() {
        let mut variable = Expression::parse("point.x".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        // EType::User(UserType::Struct(
        //     user_type_impl::Struct {
        //         id: "Point".to_string().into(),
        //         fields: {
        //             let mut res = Vec::new();
        //             res.push(("x".to_string().into(), p_num!(U64)));
        //             res.push(("y".to_string().into(), p_num!(U64)));
        //             res
        //         },
        //     }
        //     .into(),
        // ))

        let _ = scope_manager
            .register_var(
                "point",
                EType::User {
                    id: todo!(),
                    size: todo!(),
                },
                None,
            )
            .unwrap();
        let res = variable.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut None,
        );
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope_manager, None);
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(U64), variable_type)
    }

    #[test]
    fn valid_address() {
        let mut address = Address::parse("&x".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = address.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let address_type = address.type_of(&scope_manager, None);
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            EType::Static(StaticType::Address(AddrType(Box::new(p_num!(I64)))).into()),
            address_type
        )
    }

    #[test]
    fn valid_tuple() {
        let mut tuple = Tuple::parse("(1,'a')".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = tuple.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res = tuple.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Tuple(TupleType(vec![
                    p_num!(I64),
                    e_static!(StaticType::Primitive(PrimitiveType::Char)),
                ]))
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_tuple() {
        let mut tuple = Tuple::parse("(1,2)".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = tuple.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Tuple(TupleType(vec![
                    p_num!(I64),
                    e_static!(StaticType::Primitive(PrimitiveType::Char)),
                ]))
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());

        let res = tuple.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Tuple(TupleType(vec![p_num!(I64), p_num!(I64), p_num!(I64)])).into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_map() {
        let mut map = Map::parse(r##"map{string("x"):2,string("y"):6}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            map.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = map.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Map(MapType {
                    keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                    values_type: Box::new(p_num!(I64)),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_map() {
        let mut map = Map::parse(r##"map{"x":2,"y":6}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = map.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Map(MapType {
                    keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                    values_type: Box::new(EType::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());

        let res = map.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &Some(EType::Static(
                StaticType::Map(MapType {
                    keys_type: Box::new(p_num!(I64)),
                    values_type: Box::new(p_num!(I64)),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_struct() {
        let mut object = Struct::parse(r##"Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_type(
                "Point",
                UserType::Struct(user_type_impl::Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(I64)));
                        res.push(("y".to_string().into(), p_num!(I64)));
                        res
                    },
                }),
                None,
            )
            .unwrap();

        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_struct() {
        let mut object = Struct::parse(r##"Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
        let _ = scope_manager
            .register_type(
                "Point",
                UserType::Struct(user_type_impl::Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(I64)));
                        res.push((
                            "y".to_string().into(),
                            e_static!(StaticType::Primitive(PrimitiveType::Char)),
                        ));
                        res
                    },
                }),
                None,
            )
            .unwrap();

        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_union() {
        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_type(
                "Geo",
                UserType::Union(user_type_impl::Union {
                    id: "Geo".to_string().into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".to_string().into(),
                            user_type_impl::Struct {
                                id: "Point".to_string().into(),
                                fields: vec![
                                    ("x".to_string().into(), p_num!(U64)),
                                    ("y".to_string().into(), p_num!(U64)),
                                ],
                            },
                        ));
                        res.push((
                            "Axe".to_string().into(),
                            user_type_impl::Struct {
                                id: "Axe".to_string().into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push(("x".to_string().into(), p_num!(U64)));
                                    res
                                },
                            },
                        ));
                        res
                    },
                }),
                None,
            )
            .unwrap();

        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_union() {
        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();

        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());

        let _ = scope_manager
            .register_type(
                "Geo",
                UserType::Union(user_type_impl::Union {
                    id: "Geo".to_string().into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".to_string().into(),
                            user_type_impl::Struct {
                                id: "Point".to_string().into(),
                                fields: vec![
                                    ("x".to_string().into(), p_num!(U64)),
                                    (
                                        "y".to_string().into(),
                                        EType::Static(
                                            StaticType::Primitive(PrimitiveType::Char).into(),
                                        ),
                                    ),
                                ],
                            },
                        ));
                        res
                    },
                }),
                None,
            )
            .unwrap();
        let mut object = Union::parse(r##"Geo::Axe { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());

        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_enum() {
        let mut object = Enum::parse(r##"Geo::Point"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_type(
                "Geo",
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Point".to_string().into());
                        res
                    },
                }),
                None,
            )
            .unwrap();
        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_enum() {
        let mut object = Enum::parse(r##"Geo::Point"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_type(
                "Geo",
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Axe".to_string().into());
                        res
                    },
                }),
                None,
            )
            .unwrap();
        let res = object.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }
}
