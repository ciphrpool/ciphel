use nom_supreme::context;

use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData, ListAccess, Map,
    MultiData, NumAccess, Number, Primitive, PtrAccess, Slice, StrSlice, Struct, Tuple, Union,
    VarID, Variable, Vector,
};
use crate::resolve_metadata;
use crate::semantic::scope::static_types::{AddrType, NumberType, PrimitiveType, RangeType};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::{BuildVar, ClosureState};
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var, ScopeApi},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MutRc};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Data<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope, context, extra),
            Data::Slice(value) => value.resolve(scope, context, extra),
            Data::Vec(value) => value.resolve(scope, context, extra),
            Data::Closure(value) => value.resolve(scope, context, extra),
            Data::Tuple(value) => value.resolve(scope, context, extra),
            Data::Address(value) => value.resolve(scope, context, extra),
            Data::PtrAccess(value) => value.resolve(scope, context, extra),
            Data::Variable(value) => value.resolve(scope, context, extra),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope, context, extra),
            Data::Struct(value) => value.resolve(scope, context, extra),
            Data::Union(value) => value.resolve(scope, context, extra),
            Data::Enum(value) => value.resolve(scope, context, extra),
            Data::StrSlice(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<InnerScope: ScopeApi> Variable<InnerScope> {
    fn resolve_based(
        &self,
        scope: &MutRc<InnerScope>,
        context: &EType,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
        InnerScope: ScopeApi,
    {
        match self {
            Variable::Var(VarID {
                id: value,
                metadata,
            }) => {
                let var_type = <EType as GetSubTypes>::get_field(context, value)
                    .ok_or(SemanticError::UnknownField)?;
                {
                    let mut borrowed_metadata = metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
                        context: Some(var_type.clone()),
                        signature: Some(var_type.clone()),
                    };
                }
                Ok(var_type)
            }
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata,
            }) => {
                let var_type = var.resolve_based(scope, context)?;
                let field_type = field.resolve_based(scope, &var_type)?;
                {
                    let mut borrowed_metadata = metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;
                    *borrowed_metadata = Info::Resolved {
                        context: Some(field_type.clone()),
                        signature: Some(field_type.clone()),
                    };
                }
                Ok(field_type)
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata,
            }) => {
                let var_type = var.resolve_based(scope, context)?;
                if !<EType as TypeChecking>::is_indexable(&var_type) {
                    Err(SemanticError::ExpectedIterable)
                } else {
                    let _ = index.resolve(
                        scope,
                        &Some(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        )),
                        &(),
                    )?;
                    let Some(item_type) = var_type.get_item() else {
                        return Err(SemanticError::ExpectedIndexable);
                    };
                    {
                        let mut borrowed_metadata = metadata
                            .info
                            .as_ref()
                            .try_borrow_mut()
                            .map_err(|_| SemanticError::Default)?;
                        *borrowed_metadata = Info::Resolved {
                            context: Some(item_type.clone()),
                            signature: Some(item_type.clone()),
                        };
                    }
                    Ok(item_type)
                }
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata,
            }) => {
                let _ = var.resolve_based(scope, context)?;
                let var_type = var.typeof_based::<InnerScope>(&context)?;

                if !<EType as TypeChecking>::is_dotnum_indexable(&var_type) {
                    Err(SemanticError::ExpectedIndexable)
                } else {
                    let Some(fields) = <EType as GetSubTypes>::get_fields(&var_type) else {
                        return Err(SemanticError::InvalidPattern);
                    };
                    if index >= &fields.len() {
                        Err(SemanticError::InvalidPattern)
                    } else {
                        {
                            let mut borrowed_metadata = metadata
                                .info
                                .as_ref()
                                .try_borrow_mut()
                                .map_err(|_| SemanticError::Default)?;
                            *borrowed_metadata = Info::Resolved {
                                context: Some(fields[*index].1.clone()),
                                signature: Some(fields[*index].1.clone()),
                            };
                        }
                        Ok(fields[*index].1.clone())
                    }
                }
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Variable<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(value) => value.resolve(scope, context, extra),
            Variable::FieldAccess(value) => value.resolve(scope, context, extra),
            Variable::ListAccess(value) => value.resolve(scope, context, extra),
            Variable::NumAccess(value) => value.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for VarID {
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
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FieldAccess<Scope> {
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
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;
        let _ = self.field.resolve_based(scope, &var_type)?;

        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for NumAccess<Scope> {
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
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;
        if !<EType as TypeChecking>::is_dotnum_indexable(&var_type) {
            Err(SemanticError::ExpectedIndexable)
        } else {
            let Some(fields) = <EType as GetSubTypes>::get_fields(&var_type) else {
                return Err(SemanticError::InvalidPattern);
            };
            if self.index >= fields.len() {
                Err(SemanticError::InvalidPattern)
            } else {
                resolve_metadata!(self.metadata, self, scope, context);
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ListAccess<Scope> {
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
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;

        if <EType as TypeChecking>::is_indexable(&var_type) {
            let _ = self.index.resolve(
                scope,
                &Some(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                )),
                extra,
            )?;
            let index_type = self.index.type_of(&scope.borrow())?;
            if <EType as TypeChecking>::is_u64(&index_type) {
                resolve_metadata!(self.metadata, self, scope, context);
                Ok(())
            } else {
                Err(SemanticError::ExpectedIndexable)
            }
        } else {
            Err(SemanticError::ExpectedIndexable)
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Primitive {
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
        Scope: ScopeApi,
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
impl<Scope: ScopeApi> Resolve<Scope> for Slice<Scope> {
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
        Scope: ScopeApi,
    {
        let (param_context, maybe_length) = match context {
            Some(context) => (
                <EType as GetSubTypes>::get_item(context),
                <EType as GetSubTypes>::get_length(context),
            ),
            None => (None, None),
        };
        match maybe_length {
            Some(length) => {
                if length != self.value.len() {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
            None => {}
        }
        for expr in &self.value {
            let _ = expr.resolve(scope, &param_context, &())?;
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StrSlice {
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
        Scope: ScopeApi,
    {
        match context {
            Some(context_type) => {
                let _ = context_type.compatible_with(self, &scope.borrow())?;
            }
            None => {}
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Vector<Scope> {
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
        Scope: ScopeApi,
    {
        let param_context = match context {
            Some(context) => <EType as GetSubTypes>::get_item(context),
            None => None,
        };

        for expr in &self.value {
            let _ = expr.resolve(scope, &param_context, &())?;
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
impl<Scope: ScopeApi> Resolve<Scope> for Tuple<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for MultiData<Scope> {
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
        Scope: ScopeApi,
    {
        let maybe_length = match context {
            Some(context) => <EType as GetSubTypes>::get_length(context),
            None => None,
        };
        match maybe_length {
            Some(length) => {
                if length != self.len() {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
            None => {}
        }
        for (index, expr) in self.iter().enumerate() {
            let param_context = match context {
                Some(context) => <EType as GetSubTypes>::get_nth(context, &index),
                None => None,
            };
            let _ = expr.resolve(scope, &param_context, &())?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure<Scope> {
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
        Scope: ScopeApi,
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
                for (index, expr) in self.params.iter().enumerate() {
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
                let var = <Var as BuildVar<Scope>>::build_var(id, &param_type.unwrap());
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
impl<Scope: ScopeApi> Resolve<Scope> for ExprScope<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            ExprScope::Scope(value) => value.resolve(scope, context, extra),
            ExprScope::Expr(value) => value.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureParam {
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
        Scope: ScopeApi,
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
impl<Scope: ScopeApi> Resolve<Scope> for Address<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PtrAccess<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;

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
                        signature: Some(Either::Static(StaticType::Any.into())),
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
                    signature: Some(Either::Static(StaticType::Any.into())),
                };
            }
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Struct<Scope> {
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
        Scope: ScopeApi,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.id)?;
        let user_type = user_type.type_of(&scope.borrow())?;
        for (field_name, expr) in &self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&user_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &())?;
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
impl<Scope: ScopeApi> Resolve<Scope> for Union<Scope> {
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
        Scope: ScopeApi,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let variant_type = user_type.get_variant(&self.variant);
        let Some(variant_type) = variant_type else {
            return Err(SemanticError::CantInferType);
        };
        for (field_name, expr) in &self.fields {
            let field_context = <EType as GetSubTypes>::get_field(&variant_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &())?;
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

impl<Scope: ScopeApi> Resolve<Scope> for Enum {
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
        Scope: ScopeApi,
    {
        let borrowed_scope = scope.borrow();
        let user_type = borrowed_scope.find_type(&self.typename)?;
        let Some(_) = user_type.get_variant(&self.value) else {
            return Err(SemanticError::IncorrectVariant);
        };
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
        // user_type.compatible_with(&(&self.typename, &self.value), scope)?;
        // Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Map<Scope> {
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
        Scope: ScopeApi,
    {
        let item_type = match context {
            Some(context) => <EType as GetSubTypes>::get_item(context),
            None => None,
        };

        let key_type = match context {
            Some(context) => <EType as GetSubTypes>::get_key(context),
            None => None,
        };
        for (key, value) in &self.fields {
            let _ = key.resolve(scope, &key_type, &())?;
            let _ = value.resolve(scope, &item_type, &())?;
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
impl<Scope: ScopeApi> Resolve<Scope> for KeyData<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            KeyData::Address(value) => value.resolve(scope, context, extra),
            KeyData::Enum(value) => value.resolve(scope, context, extra),
            KeyData::Primitive(value) => value.resolve(scope, context, extra),
            KeyData::StrSlice(value) => value.resolve(scope, context, extra),
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
        ast::{statements, TryParse},
        semantic::scope::{
            scope_impl::Scope,
            static_types::{
                AddrType, ChanType, KeyType, MapType, NumberType, PrimitiveType, SliceType,
                StaticType, StrSliceType, StringType, TupleType, VecType,
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

        let res = primitive.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
            )),
            &(),
        );
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
                    item_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    )),
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

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Slice(SliceType {
                    size: 4,
                    item_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
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
                StaticType::Vec(VecType(Box::new(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ))))
                .into(),
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            variable_type
        );
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
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
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
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn robustness_variable_array() {
        let variable = Variable::parse("x[\"Test\"]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::String(StringType()),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                    })
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
    #[test]
    fn valid_variable_tuple() {
        let variable = Variable::parse("x.0".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Tuple(TupleType(vec![
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        ),
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        ),
                    ]))
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            variable_type
        );
    }
    #[test]
    fn valid_variable_struct() {
        let variable = Variable::parse("point.x".into()).unwrap().1;
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
                                res.push((
                                    "x".into(),
                                    Either::Static(
                                        StaticType::Primitive(PrimitiveType::Number(
                                            NumberType::U64,
                                        ))
                                        .into(),
                                    ),
                                ));
                                res.push((
                                    "y".into(),
                                    Either::Static(
                                        StaticType::Primitive(PrimitiveType::Number(
                                            NumberType::U64,
                                        ))
                                        .into(),
                                    ),
                                ));
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
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            variable_type
        )
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = address.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let address_type = address.type_of(&scope.borrow());
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            Either::Static(
                StaticType::Address(AddrType(Box::new(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()
                ))))
                .into()
            ),
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
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                    Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                    Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                ]))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());

        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Tuple(TupleType(vec![
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ),
                ]))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_map() {
        let map = Map::parse(r##"map{"x":2,"y":6}"##.into()).unwrap().1;
        let scope = Scope::new();
        let res = map.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::String(StringType()),
                    values_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    )),
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
                    keys_type: KeyType::String(StringType()),
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
                    keys_type: KeyType::Primitive(PrimitiveType::Number(NumberType::I64)),
                    values_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    )),
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
                        res.push((
                            "x".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                        ));
                        res.push((
                            "y".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
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
                        res.push((
                            "x".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                        ));
                        res.push((
                            "y".into(),
                            Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
                                fields: vec![
                                    (
                                        "x".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ),
                                    (
                                        "y".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ),
                                ],
                            },
                        ));
                        res.push((
                            "Axe".into(),
                            user_type_impl::Struct {
                                id: "Axe".into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push((
                                        "x".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ));
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
                                    (
                                        "x".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ),
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
