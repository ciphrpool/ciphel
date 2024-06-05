
use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, Map, Number, Primitive,
    PtrAccess, Slice, StrSlice, Struct, Tuple, Union, Variable, Vector,
};
use crate::ast::expressions::Atomic;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{
    AddrType, MapType, NumberType, PrimitiveType, SliceType, StrSliceType, TupleType, VecType,
};
use crate::semantic::scope::type_traits::{GetSubTypes};
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::{BuildVar, ClosureState};
use crate::semantic::{
    scope::{static_types::StaticType, var_impl::Var},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MergeType};
use crate::{arw_read, e_static, resolve_metadata};

impl Resolve for Data {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope, context, &mut ()),
            Data::Slice(value) => value.resolve(scope, context, &mut ()),
            Data::Vec(value) => value.resolve(scope, context, &mut ()),
            Data::Closure(value) => value.resolve(scope, context, &mut ()),
            Data::Tuple(value) => value.resolve(scope, context, &mut ()),
            Data::Address(value) => value.resolve(scope, context, &mut ()),
            Data::PtrAccess(value) => value.resolve(scope, context, &mut ()),
            Data::Variable(value) => value.resolve(scope, context, extra),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope, context, &mut ()),
            Data::Struct(value) => value.resolve(scope, context, &mut ()),
            Data::Union(value) => value.resolve(scope, context, &mut ()),
            Data::Enum(value) => value.resolve(scope, context, &mut ()),
            Data::StrSlice(value) => value.resolve(scope, context, &mut ()),
        }
    }
}

impl Resolve for Variable {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match extra {
            Some(extra) => {
                self.from_field = true;
                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(extra.clone()),
                };
            }
            None => {
                let var =
                    crate::arw_read!(scope, SemanticError::ConcurrencyError)?.find_var(&self.id)?;
                let var_type = arw_read!(var, SemanticError::ConcurrencyError)?
                    .type_sig
                    .clone();

                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(var_type),
                };
            }
        }
        Ok(())
    }
}

impl Resolve for Primitive {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(context_type) => {
                match context_type {
                    Either::Static(value) => {
                        if let Primitive::Number(n) = self {
                            if let Number::Unresolved(v) = n {
                                match value.as_ref() {
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
                    Either::User(_) => {}
                }
                let _ = context_type.compatible_with(
                    self,
                    &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
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
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                Either::Static(rcontext) => match rcontext.as_ref() {
                    StaticType::Slice(SliceType { size, item_type }) => {
                        let item_type = item_type.as_ref().clone();
                        let sitem_type = &Some(item_type.clone());
                        if self.value.len() > *size {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                        for value in &mut self.value {
                            let _ = value.resolve(scope, sitem_type, &mut None)?;
                            let _ = item_type.compatible_with(
                                value,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
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
                Either::User(_) => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut item_type = e_static!(StaticType::Any);
                for value in &mut self.value {
                    let _ = value.resolve(scope, &None, &mut None)?;
                    item_type = item_type.merge(
                        value,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                }

                let sitem_type = Some(item_type.clone());
                for value in &mut self.value {
                    let _ = value.resolve(scope, &sitem_type, &mut None)?;
                }

                {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Slice(SliceType {
                            size: self.value.len(),
                            item_type: Box::new(item_type),
                        }))),
                    };
                }
                Ok(())
            }
        }
    }
}

impl Resolve for StrSlice {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(context_type) => match context_type {
                Either::Static(value) => match value.as_ref() {
                    StaticType::StrSlice(StrSliceType { size }) => {
                        if *size < self.value.len() {
                            return Err(SemanticError::IncompatibleTypes);
                        } else if *size >= self.value.len() {
                            self.padding = *size - self.value.len();
                        }
                    }
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
                Either::User(_) => return Err(SemanticError::IncompatibleTypes),
            },
            None => {}
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Vector {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                Either::Static(rcontext) => match rcontext.as_ref() {
                    StaticType::Vec(VecType(item_type)) => {
                        let item_type = item_type.as_ref().clone();
                        let sitem_type = &Some(item_type.clone());

                        for value in &mut self.value {
                            let _ = value.resolve(scope, sitem_type, &mut None)?;
                            let _ = item_type.compatible_with(
                                value,
                                &&crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;
                        }

                        {
                            self.metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(e_static!(StaticType::Vec(VecType(Box::new(
                                    item_type
                                ),)))),
                            };
                        }
                        Ok(())
                    }
                    _ => Err(SemanticError::IncompatibleTypes),
                },
                Either::User(_) => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut item_type = e_static!(StaticType::Any);
                for value in &mut self.value {
                    let _ = value.resolve(scope, &None, &mut None)?;
                    item_type = item_type.merge(
                        value,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                }

                let sitem_type = Some(item_type.clone());
                for value in &mut self.value {
                    let _ = value.resolve(scope, &sitem_type, &mut None)?;
                }

                {
                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Vec(VecType(Box::new(item_type),)))),
                    };
                }
                Ok(())
            }
        }
    }
}
impl Resolve for Tuple {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                Either::Static(rcontext) => match rcontext.as_ref() {
                    StaticType::Tuple(TupleType(values_type)) => {
                        if values_type.len() != self.value.len() {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                        for (value, value_type) in self.value.iter_mut().zip(values_type) {
                            let _ = value.resolve(scope, &Some(value_type.clone()), &mut None)?;
                            let _ = value_type.compatible_with(
                                value,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
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
                Either::User(_) => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut values_type = Vec::new();
                for value in &mut self.value {
                    let _ = value.resolve(scope, &None, &mut None)?;
                    values_type.push(
                        value.type_of(
                            &(crate::arw_read!(scope, SemanticError::ConcurrencyError))?,
                        )?,
                    );
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
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        // let Some(context) = context else {
        //     return Err(SemanticError::CantInferType);
        // };
        match context {
            Some(context) => {
                for (index, expr) in self.params.iter_mut().enumerate() {
                    let param_context = <EType as GetSubTypes>::get_nth(context, &index);
                    let _ = expr.resolve(scope, &param_context, &mut ())?;
                }
            }
            None => {
                for (_index, expr) in self.params.iter_mut().enumerate() {
                    let _ = expr.resolve(scope, &None, &mut ())?;
                }
            }
        }
        let borrowed_scope = &crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
        let mut vars = self
            .params
            .iter()
            .enumerate()
            .filter_map(|(idx, param)| {
                param.type_of(borrowed_scope).ok().map(|p| {
                    (
                        idx,
                        match param {
                            ClosureParam::Full(var) => &var.id,
                            ClosureParam::Minimal(id) => id,
                        },
                        p,
                    )
                })
            })
            .map(|(index, id, param)| {
                let param_type = match context {
                    Some(context) => <EType as GetSubTypes>::get_nth(context, &index),
                    None => Some(param),
                };
                let mut var = <Var as BuildVar>::build_var(id, &param_type.unwrap());
                var.state = VarState::Parameter;
                var.is_declared = true;
                var
            })
            .collect::<Vec<Var>>();

        let context_return = match context {
            Some(context) => <EType as GetSubTypes>::get_return(context),
            None => None,
        };
        if self.closed {
            self.scope.to_capturing(ClosureState::CAPTURING);
        } else {
            self.scope.to_capturing(ClosureState::NOT_CAPTURING);
        }

        let _ = self.scope.resolve(scope, &context_return, &mut vars)?;
        if !self.closed
            && self
                .scope
                .scope()?
                .try_read()
                .map_err(|_| SemanticError::ConcurrencyError)?
                .env_vars()?
                .len()
                > 0
        {
            return Err(SemanticError::ExpectedMovedClosure);
        }
        {
            self.metadata.info = Info::Resolved {
                context: context.clone(),
                signature: Some(
                    self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?,
                ),
            };
        }
        Ok(())
    }
}
impl Resolve for ExprScope {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Var>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ExprScope::Scope(value) => value.resolve(scope, context, extra),
            ExprScope::Expr(value) => value.resolve(scope, context, extra),
        }
    }
}
impl Resolve for ClosureParam {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ClosureParam::Full(var) => var.resolve(scope, &mut (), &mut ()),
            ClosureParam::Minimal(_value) => match context {
                Some(_) => Ok(()),
                None => Err(SemanticError::CantInferType),
            },
        }
    }
}
impl Resolve for Address {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
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
        let _ = self.value.resolve(scope, context, &mut None)?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for PtrAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.value.resolve(scope, context, &mut None)?;

        let address_type = self
            .value
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

        match &address_type {
            Either::Static(value) => match value.as_ref() {
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
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = &crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
        let user_type = borrowed_scope.find_type(&self.id)?;
        let user_type =
            user_type.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        for (ref field_name, expr) in &mut self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&user_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &mut None)?;
        }

        let Some(fields_type) = <EType as GetSubTypes>::get_fields(&user_type) else {
            return Err(SemanticError::ExpectedStruct);
        };
        if self.fields.len() != fields_type.len() {
            return Err(SemanticError::IncorrectStruct);
        }
        for (field_name, field_type) in fields_type {
            let Some(field_name) = field_name else {
                return Err(SemanticError::IncorrectStruct);
            };
            let Some(expr_field) = self
                .fields
                .iter()
                .find(|(name, _)| name == &field_name)
                .map(|(_, expr)| expr)
            else {
                return Err(SemanticError::IncorrectStruct);
            };
            let _ = field_type.compatible_with(
                expr_field,
                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
            )?;
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for Union {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let variant_type = user_type.get_variant(&self.variant);
        let Some(variant_type) = variant_type else {
            return Err(SemanticError::CantInferType);
        };
        for (field_name, expr) in &mut self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&variant_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &mut None)?;
        }

        let Some(fields_type) = <EType as GetSubTypes>::get_fields(&variant_type) else {
            return Err(SemanticError::ExpectedStruct);
        };
        if self.fields.len() != fields_type.len() {
            return Err(SemanticError::IncorrectStruct);
        }
        for (field_name, field_type) in fields_type {
            let Some(field_name) = field_name else {
                return Err(SemanticError::IncorrectStruct);
            };
            let Some(expr_field) = self
                .fields
                .iter()
                .find(|(name, _)| name == &field_name)
                .map(|(_, expr)| expr)
            else {
                return Err(SemanticError::IncorrectStruct);
            };
            let _ = field_type.compatible_with(
                expr_field,
                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
            )?;
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Enum {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let Some(_) = user_type.get_variant(&self.value) else {
            return Err(SemanticError::IncorrectVariant);
        };
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
        // user_type.compatible_with(&(&self.typename, &self.value), block)?;
        // Ok(())
    }
}

impl Resolve for Map {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(rcontext) => match rcontext {
                Either::Static(rcontext) => match rcontext.as_ref() {
                    StaticType::Map(MapType {
                        keys_type,
                        values_type,
                    }) => {
                        let key_type = keys_type.as_ref().clone();
                        let skey_type = &Some(key_type);
                        let value_type = values_type.as_ref().clone();
                        let svalue_type = &Some(value_type);

                        for (key, value) in &mut self.fields {
                            let _ = key.resolve(scope, skey_type, &mut None)?;
                            let _ = keys_type.compatible_with(
                                key,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;
                            let _ = value.resolve(scope, svalue_type, &mut None)?;
                            let _ = values_type.compatible_with(
                                value,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
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
                Either::User(_) => Err(SemanticError::IncompatibleTypes),
            },
            None => {
                let mut keys_type = e_static!(StaticType::Any);
                let mut values_type = e_static!(StaticType::Any);
                for (key, value) in &mut self.fields {
                    let _ = key.resolve(scope, &None, &mut None)?;
                    keys_type = keys_type.merge(
                        key,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                    let _ = value.resolve(scope, &None, &mut None)?;
                    values_type = values_type.merge(
                        value,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                }

                let skeys_type = Some(keys_type.clone());
                let svalues_type = Some(values_type.clone());
                for (key, value) in &mut self.fields {
                    let _ = key.resolve(scope, &skeys_type, &mut None)?;
                    let _ = value.resolve(scope, &svalues_type, &mut None)?;
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
        arw_write,
        ast::{expressions::Expression, TryParse},
        p_num,
        semantic::scope::{
            scope::Scope,
            static_types::{
                AddrType, MapType, PrimitiveType, SliceType, StaticType, StrSliceType,
                StringType, TupleType, VecType,
            },
            user_type_impl::{self, UserType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_primitive() {
        let mut primitive = Primitive::parse("1".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let res = primitive.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = primitive.resolve(&scope, &Some(p_num!(I64)), &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_primitive() {
        let mut primitive = Primitive::parse("1".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();

        let res = primitive.resolve(
            &scope,
            &Some(Either::Static(
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

        let scope = Scope::new();
        let res = string.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = string.resolve(
            &scope,
            &Some(Either::Static(
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

        let scope = Scope::new();
        let res = slice.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();

        let res = string.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Slice(SliceType {
                    size: 2,
                    item_type: Box::new(Either::Static(
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

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = vector.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();

        let res = vector.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Vec(VecType(Box::new(Either::Static(
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
        let scope = Scope::new();
        let _ = arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type =
            variable.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_array() {
        let mut variable = Variable::parse("x[10]".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::Static(StaticType::Vec(VecType(Box::new(p_num!(I64)))).into()),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_variable_array_complex() {
        let mut variable = Variable::parse("x[10 + 10]".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::Static(StaticType::Vec(VecType(Box::new(p_num!(I64)))).into()),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_variable_array() {
        let mut variable = Expression::parse("x[\"Test\"]".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                        values_type: Box::new(p_num!(I64)),
                    })
                    .into(),
                ),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }
    #[test]
    fn valid_variable_tuple() {
        let mut variable = Expression::parse("x.0".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::Static(
                    StaticType::Tuple(TupleType(vec![p_num!(I64), p_num!(I64)])).into(),
                ),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type =
            variable.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_struct() {
        let mut variable = Expression::parse("point.x".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "point".to_string().into(),
                type_sig: Either::User(
                    UserType::Struct(
                        user_type_impl::Struct {
                            id: "Point".to_string().into(),
                            fields: {
                                let mut res = Vec::new();
                                res.push(("x".to_string().into(), p_num!(U64)));
                                res.push(("y".to_string().into(), p_num!(U64)));
                                res
                            },
                        }
                        .into(),
                    )
                    .into(),
                ),
                is_declared: false,
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type =
            variable.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(U64), variable_type)
    }

    #[test]
    fn valid_address() {
        let mut address = Address::parse("&x".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .unwrap();
        let res = address.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let address_type =
            address.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap());
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Address(AddrType(Box::new(p_num!(I64)))).into()),
            address_type
        )
    }

    #[test]
    fn valid_tuple() {
        let mut tuple = Tuple::parse("(1,'a')".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = tuple.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();
        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Tuple(TupleType(vec![
                    p_num!(I64),
                    e_static!(StaticType::Primitive(PrimitiveType::Char)),
                ]))
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());

        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
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
        dbg!(&map);
        let scope = Scope::new();
        let res = map.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Map(MapType {
                    keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                    values_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                })
                .into(),
            )),
            &mut (),
        );
        assert!(res.is_err());

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
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
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(user_type_impl::Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(I64)));
                        res.push(("y".to_string().into(), p_num!(I64)));
                        res
                    },
                }),
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_struct() {
        let mut object = Struct::parse(r##"Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
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
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_union() {
        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Geo".to_string().into(),
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
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_union() {
        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();

        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());

        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Geo".to_string().into(),
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
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Char).into(),
                                        ),
                                    ),
                                ],
                            },
                        ));
                        res
                    },
                }),
            )
            .unwrap();
        let mut object = Union::parse(r##"Geo::Axe { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());

        let mut object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_enum() {
        let mut object = Enum::parse(r##"Geo::Point"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Geo".to_string().into(),
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Point".to_string().into());
                        res
                    },
                }),
            )
            .unwrap();
        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_enum() {
        let mut object = Enum::parse(r##"Geo::Point"##.into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Geo".to_string().into(),
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Axe".to_string().into());
                        res
                    },
                }),
            )
            .unwrap();
        let res = object.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }
}
