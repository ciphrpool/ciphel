use nom_supreme::context;

use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, Map, MultiData, Number, Primitive,
    PtrAccess, Slice, StrSlice, Struct, Tuple, Union, Variable, Vector,
};
use crate::ast::expressions::Atomic;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{
    AddrType, MapType, NumberType, PrimitiveType, SliceType, StrSliceType, TupleType, VecType,
};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::{BuildVar, ClosureState};
use crate::semantic::{
    scope::{static_types::StaticType, var_impl::Var},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MergeType, MutRc};
use crate::{e_static, p_num, resolve_metadata};

impl Resolve for Data {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope, context, &()),
            Data::Slice(value) => value.resolve(scope, context, &()),
            Data::Vec(value) => value.resolve(scope, context, &()),
            Data::Closure(value) => value.resolve(scope, context, &()),
            Data::Tuple(value) => value.resolve(scope, context, &()),
            Data::Address(value) => value.resolve(scope, context, &()),
            Data::PtrAccess(value) => value.resolve(scope, context, &()),
            Data::Variable(value) => value.resolve(scope, context, extra),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope, context, &()),
            Data::Struct(value) => value.resolve(scope, context, &()),
            Data::Union(value) => value.resolve(scope, context, &()),
            Data::Enum(value) => value.resolve(scope, context, &()),
            Data::StrSlice(value) => value.resolve(scope, context, &()),
        }
    }
}

impl Resolve for Variable {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match extra {
            Some(extra) => {
                self.from_field.set(true);
                let mut borrowed_metadata = self
                    .metadata
                    .info
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| SemanticError::Default)?;
                *borrowed_metadata = Info::Resolved {
                    context: context.clone(),
                    signature: Some(extra.clone()),
                };
            }
            None => {
                let var = scope.borrow().find_var(&self.id)?;
                let var_type = var.type_sig.clone();

                let mut borrowed_metadata = self
                    .metadata
                    .info
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| SemanticError::Default)?;
                *borrowed_metadata = Info::Resolved {
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match context {
            Some(context_type) => {
                match context_type {
                    Either::Static(value) => {
                        if let Primitive::Number(n) = self {
                            if let Number::Unresolved(v) = n.get() {
                                match value.as_ref() {
                                    StaticType::Primitive(PrimitiveType::Number(value)) => {
                                        match value {
                                            NumberType::U8 => {
                                                n.set(Number::U8(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::U16 => {
                                                n.set(Number::U16(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::U32 => {
                                                n.set(Number::U32(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::U64 => {
                                                n.set(Number::U64(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::U128 => {
                                                n.set(Number::U128(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::I8 => {
                                                n.set(Number::I8(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::I16 => {
                                                n.set(Number::I16(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::I32 => {
                                                n.set(Number::I32(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::I64 => {
                                                n.set(Number::I64(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::I128 => {
                                                n.set(Number::I128(v.try_into().map_err(|_| {
                                                    SemanticError::IncompatibleTypes
                                                })?))
                                            }
                                            NumberType::F64 => n.set(Number::F64(v as f64)),
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Either::User(_) => {}
                }
                let _ = context_type.compatible_with(self, &scope.borrow())?;
                Ok(())
            }
            None => {
                if let Primitive::Number(n) = self {
                    if let Number::Unresolved(v) = n.get() {
                        n.set(Number::I64(v))
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
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
                        for value in &self.value {
                            let _ = value.resolve(scope, sitem_type, &None)?;
                            let _ = item_type.compatible_with(value, &scope.borrow())?;
                        }

                        {
                            let mut borrowed_metadata = self
                                .metadata
                                .info
                                .as_ref()
                                .try_borrow_mut()
                                .map_err(|_| SemanticError::Default)?;
                            *borrowed_metadata = Info::Resolved {
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
                for value in &self.value {
                    let _ = value.resolve(scope, &None, &None)?;
                    item_type = item_type.merge(value, &scope.borrow())?;
                }

                let sitem_type = Some(item_type.clone());
                for value in &self.value {
                    let _ = value.resolve(scope, &sitem_type, &None)?;
                }

                {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
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
                            self.padding.set(*size - self.value.len());
                        }
                    }
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
                Either::User(_) => return Err(SemanticError::IncompatibleTypes),
            },
            None => {}
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Vector {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
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

                        for value in &self.value {
                            let _ = value.resolve(scope, sitem_type, &None)?;
                            let _ = item_type.compatible_with(value, &scope.borrow())?;
                        }

                        {
                            let mut borrowed_metadata = self
                                .metadata
                                .info
                                .as_ref()
                                .try_borrow_mut()
                                .map_err(|_| SemanticError::Default)?;
                            *borrowed_metadata = Info::Resolved {
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
                for value in &self.value {
                    let _ = value.resolve(scope, &None, &None)?;
                    item_type = item_type.merge(value, &scope.borrow())?;
                }

                let sitem_type = Some(item_type.clone());
                for value in &self.value {
                    let _ = value.resolve(scope, &sitem_type, &None)?;
                }

                {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
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
                        for (value, value_type) in self.value.iter().zip(values_type) {
                            let _ = value.resolve(scope, &Some(value_type.clone()), &None)?;
                            let _ = value_type.compatible_with(value, &scope.borrow())?;
                        }

                        {
                            let mut borrowed_metadata = self
                                .metadata
                                .info
                                .as_ref()
                                .try_borrow_mut()
                                .map_err(|_| SemanticError::Default)?;
                            *borrowed_metadata = Info::Resolved {
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
                for value in &self.value {
                    let _ = value.resolve(scope, &None, &None)?;
                    values_type.push(value.type_of(&scope.borrow())?);
                }

                {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        // let Some(context) = context else {
        //     return Err(SemanticError::CantInferType);
        // };
        match context {
            Some(context) => {
                for (index, expr) in self.params.iter().enumerate() {
                    let param_context = <EType as GetSubTypes>::get_nth(context, &index);
                    let _ = expr.resolve(scope, &param_context, &())?;
                }
            }
            None => {
                for (_index, expr) in self.params.iter().enumerate() {
                    let _ = expr.resolve(scope, &None, &())?;
                }
            }
        }

        let vars = self
            .params
            .iter()
            .enumerate()
            .filter_map(|(idx, param)| {
                param.type_of(&scope.borrow()).ok().map(|p| {
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
                let var = <Var as BuildVar>::build_var(id, &param_type.unwrap());
                var.state.set(VarState::Parameter);
                var.is_declared.set(true);
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

        let _ = self.scope.resolve(scope, &context_return, &vars)?;
        if !self.closed && self.scope.scope()?.borrow().env_vars().len() > 0 {
            return Err(SemanticError::ExpectedMovedClosure);
        }
        {
            let mut borrowed_metadata = self
                .metadata
                .info
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| SemanticError::Default)?;

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ClosureParam::Full(var) => var.resolve(scope, &(), &()),
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self.value.as_ref() {
            Atomic::Data(_) => {}
            Atomic::UnaryOperation(_) => return Err(SemanticError::IncompatibleTypes),
            Atomic::Paren(_) => {}
            Atomic::ExprFlow(_) => return Err(SemanticError::IncompatibleTypes),
            Atomic::Error(_) => {}
        }
        let _ = self.value.resolve(scope, context, &None)?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for PtrAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.value.resolve(scope, context, &None)?;

        let address_type = self.value.type_of(&scope.borrow())?;

        match &address_type {
            Either::Static(value) => match value.as_ref() {
                StaticType::Address(AddrType(sub)) => {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(sub.as_ref().clone()),
                    };
                }
                _ => {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(e_static!(StaticType::Any)),
                    };
                }
            },
            _ => {
                let mut borrowed_metadata = self
                    .metadata
                    .info
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| SemanticError::Default)?;
                *borrowed_metadata = Info::Resolved {
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.id)?;
        let user_type = user_type.type_of(&scope.borrow())?;
        for (field_name, expr) in &self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&user_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &None)?;
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
            let _ = field_type.compatible_with(expr_field, &scope.borrow())?;
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for Union {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let variant_type = user_type.get_variant(&self.variant);
        let Some(variant_type) = variant_type else {
            return Err(SemanticError::CantInferType);
        };
        for (field_name, expr) in &self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&variant_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &None)?;
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
            let _ = field_type.compatible_with(expr_field, &scope.borrow())?;
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Enum {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let Some(_) = user_type.get_variant(&self.value) else {
            return Err(SemanticError::IncorrectVariant);
        };
        resolve_metadata!(self.metadata, self, scope, context);
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
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
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

                        for (key, value) in &self.fields {
                            let _ = key.resolve(scope, skey_type, &None)?;
                            let _ = keys_type.compatible_with(key, &scope.borrow())?;
                            let _ = value.resolve(scope, svalue_type, &None)?;
                            let _ = values_type.compatible_with(value, &scope.borrow())?;
                        }

                        {
                            let mut borrowed_metadata = self
                                .metadata
                                .info
                                .as_ref()
                                .try_borrow_mut()
                                .map_err(|_| SemanticError::Default)?;
                            *borrowed_metadata = Info::Resolved {
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
                for (key, value) in &self.fields {
                    let _ = key.resolve(scope, &None, &None)?;
                    keys_type = keys_type.merge(key, &scope.borrow())?;
                    let _ = value.resolve(scope, &None, &None)?;
                    values_type = values_type.merge(value, &scope.borrow())?;
                }

                let skeys_type = Some(keys_type.clone());
                let svalues_type = Some(values_type.clone());
                for (key, value) in &self.fields {
                    let _ = key.resolve(scope, &skeys_type, &None)?;
                    let _ = value.resolve(scope, &svalues_type, &None)?;
                }

                {
                    let mut borrowed_metadata = self
                        .metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
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
    use std::{
        cell::Cell,
        collections::{HashMap, HashSet},
    };

    use crate::{
        ast::{expressions::Expression, statements, TryParse},
        p_num,
        semantic::scope::{
            scope::Scope,
            static_types::{
                AddrType, ChanType, MapType, NumberType, PrimitiveType, SliceType, StaticType,
                StrSliceType, StringType, TupleType, VecType,
            },
            user_type_impl::{self, UserType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_primitive() {
        let primitive = Primitive::parse("1".into());
        assert!(primitive.is_ok());
        let primitive = primitive.unwrap().1;

        let scope = Scope::new();
        let res = primitive.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let res = primitive.resolve(&scope, &Some(p_num!(I64)), &());
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_primitive() {
        let primitive = Primitive::parse("1".into()).unwrap().1;
        let scope = Scope::new();

        let res = primitive.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_string() {
        let string = StrSlice::parse(r##""Hello World""##.into()).unwrap().1;

        let scope = Scope::new();
        let res = string.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let res = string.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::StrSlice(StrSliceType {
                    size: "Hello World".len(),
                })
                .into(),
            )),
            &(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_slice() {
        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let scope = Scope::new();
        let res = slice.resolve(&scope, &None, &());
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
            &(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_string() {
        let string = StrSlice::parse(r##""Hello World""##.into()).unwrap().1;
        let scope = Scope::new();

        let res = string.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }
    #[test]
    fn robustness_slice() {
        let slice = Slice::parse("[1,2]".into()).unwrap().1;
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
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_vector() {
        let vector = Vector::parse("vec[1,2,3]".into()).unwrap().1;

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let res = vector.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Vec(VecType(Box::new(p_num!(I64)))).into(),
            )),
            &(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_vector() {
        let vector = Vector::parse("vec[1,2,3]".into()).unwrap().1;
        let scope = Scope::new();

        let res = vector.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Vec(VecType(Box::new(Either::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                ))))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_variable() {
        let variable = Variable::parse("x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_array() {
        let variable = Variable::parse("x[10]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(StaticType::Vec(VecType(Box::new(p_num!(I64)))).into()),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_variable_array_complex() {
        let variable = Variable::parse("x[10 + 10]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(StaticType::Vec(VecType(Box::new(p_num!(I64)))).into()),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_variable_array() {
        let variable = Expression::parse("x[\"Test\"]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: Box::new(e_static!(StaticType::String(StringType()))),
                        values_type: Box::new(p_num!(I64)),
                    })
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_err());
    }
    #[test]
    fn valid_variable_tuple() {
        let variable = Expression::parse("x.0".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Tuple(TupleType(vec![p_num!(I64), p_num!(I64)])).into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(I64), variable_type);
    }
    #[test]
    fn valid_variable_struct() {
        let variable = Expression::parse("point.x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "point".into(),
                type_sig: Either::User(
                    UserType::Struct(
                        user_type_impl::Struct {
                            id: "Point".into(),
                            fields: {
                                let mut res = Vec::new();
                                res.push(("x".into(), p_num!(U64)));
                                res.push(("y".into(), p_num!(U64)));
                                res
                            },
                        }
                        .into(),
                    )
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &None);
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(p_num!(U64), variable_type)
    }

    #[test]
    fn valid_address() {
        let address = Address::parse("&x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = address.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let address_type = address.type_of(&scope.borrow());
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Address(AddrType(Box::new(p_num!(I64)))).into()),
            address_type
        )
    }

    #[test]
    fn valid_tuple() {
        let tuple = Tuple::parse("(1,'a')".into()).unwrap().1;
        let scope = Scope::new();
        let res = tuple.resolve(&scope, &None, &());
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
            &(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_tuple() {
        let tuple = Tuple::parse("(1,2)".into()).unwrap().1;
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
            &(),
        );
        assert!(res.is_err());

        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Tuple(TupleType(vec![p_num!(I64), p_num!(I64), p_num!(I64)])).into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_map() {
        let map = Map::parse(r##"map{string("x"):2,string("y"):6}"##.into())
            .unwrap()
            .1;
        dbg!(&map);
        let scope = Scope::new();
        let res = map.resolve(&scope, &None, &());
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
            &(),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_map() {
        let map = Map::parse(r##"map{"x":2,"y":6}"##.into()).unwrap().1;
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
            &(),
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
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_struct() {
        let object = Struct::parse(r##"Point { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Point".into(),
                UserType::Struct(user_type_impl::Struct {
                    id: "Point".into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".into(), p_num!(I64)));
                        res.push(("y".into(), p_num!(I64)));
                        res
                    },
                }),
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_struct() {
        let object = Struct::parse(r##"Point { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Point".into(),
                UserType::Struct(user_type_impl::Struct {
                    id: "Point".into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".into(), p_num!(I64)));
                        res.push((
                            "y".into(),
                            e_static!(StaticType::Primitive(PrimitiveType::Char)),
                        ));
                        res
                    },
                }),
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_union() {
        let object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Geo".into(),
                UserType::Union(user_type_impl::Union {
                    id: "Geo".into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".into(),
                            user_type_impl::Struct {
                                id: "Point".into(),
                                fields: vec![("x".into(), p_num!(U64)), ("y".into(), p_num!(U64))],
                            },
                        ));
                        res.push((
                            "Axe".into(),
                            user_type_impl::Struct {
                                id: "Axe".into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push(("x".into(), p_num!(U64)));
                                    res
                                },
                            },
                        ));
                        res
                    },
                }),
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_union() {
        let object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let scope = Scope::new();

        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());

        let _ = scope
            .borrow_mut()
            .register_type(
                &"Geo".into(),
                UserType::Union(user_type_impl::Union {
                    id: "Geo".into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".into(),
                            user_type_impl::Struct {
                                id: "Point".into(),
                                fields: vec![
                                    ("x".into(), p_num!(U64)),
                                    (
                                        "y".into(),
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
        let object = Union::parse(r##"Geo::Axe { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());

        let object = Union::parse(r##"Geo::Point { x : 2, y : 8}"##.into())
            .unwrap()
            .1;
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_enum() {
        let object = Enum::parse(r##"Geo::Point"##.into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Geo".into(),
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Point".into());
                        res
                    },
                }),
            )
            .unwrap();
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_enum() {
        let object = Enum::parse(r##"Geo::Point"##.into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Geo".into(),
                UserType::Enum(user_type_impl::Enum {
                    id: "Geo".into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("Axe".into());
                        res
                    },
                }),
            )
            .unwrap();
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}
