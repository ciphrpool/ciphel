use super::{
    Address, Channel, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData,
    ListAccess, Map, MultiData, NumAccess, Primitive, PtrAccess, Slice, Struct, Tuple, Union,
    VarID, Variable, Vector,
};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::Info;
use crate::semantic::{
    scope::{
        static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Data<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
            Data::Chan(value) => value.resolve(scope, context, extra),
            Data::Tuple(value) => value.resolve(scope, context, extra),
            Data::Address(value) => value.resolve(scope, context, extra),
            Data::PtrAccess(value) => value.resolve(scope, context, extra),
            Data::Variable(value) => value.resolve(scope, context, extra),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope, context, extra),
            Data::Struct(value) => value.resolve(scope, context, extra),
            Data::Union(value) => value.resolve(scope, context, extra),
            Data::Enum(value) => value.resolve(scope, context, extra),
        }
    }
}

impl<InnerScope: ScopeApi> Variable<InnerScope> {
    fn resolve_based(
        &self,
        scope: &Rc<RefCell<InnerScope>>,
        context: &Either<UserType, StaticType>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Self: Sized,
        InnerScope: ScopeApi,
    {
        match self {
            Variable::Var(VarID {
                id: value,
                metadata: _,
            }) => <Either<UserType, StaticType> as GetSubTypes>::get_field(context, value)
                .ok_or(SemanticError::UnknownField),
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata: _,
            }) => {
                let var_type = var.resolve_based(scope, context)?;
                field.resolve_based(scope, &var_type)
            }
            Variable::ListAccess(ListAccess {
                var,
                index,
                metadata: _,
            }) => {
                let var_type = var.resolve_based(scope, context)?;
                if !<Either<UserType, StaticType> as TypeChecking>::is_iterable(&var_type) {
                    Err(SemanticError::ExpectedIterable)
                } else {
                    let key_type =
                        <Either<UserType, StaticType> as GetSubTypes>::get_key(&var_type);

                    let _ = index.resolve(scope, &key_type, &())?;
                    let index_type = index.type_of(&scope.borrow())?;
                    let _ = key_type.compatible_with(&index_type, &scope.borrow())?;
                    Ok(var_type)
                }
            }
            Variable::NumAccess(NumAccess {
                var,
                index,
                metadata: _,
            }) => {
                let _ = var.resolve_based(scope, context)?;
                let var_type = var.type_of(&scope.borrow())?;
                if !<Either<UserType, StaticType> as TypeChecking>::is_indexable(&var_type) {
                    Err(SemanticError::ExpectedIndexable)
                } else {
                    let Some(fields) =
                        <Either<UserType, StaticType> as GetSubTypes>::get_fields(&var_type)
                    else {
                        return Err(SemanticError::InvalidPattern);
                    };
                    if index >= &fields.len() {
                        Err(SemanticError::InvalidPattern)
                    } else {
                        Ok(fields[*index].1.clone())
                    }
                }
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Variable<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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

    type Context = Option<Either<UserType, StaticType>>;

    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _var = scope.borrow().find_var(&self.id)?;
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();
            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FieldAccess<Scope> {
    type Output = ();

    type Context = Option<Either<UserType, StaticType>>;

    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;
        let _ = self.field.resolve_based(scope, &var_type)?;
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();
            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for NumAccess<Scope> {
    type Output = ();

    type Context = Option<Either<UserType, StaticType>>;

    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;
        if !<Either<UserType, StaticType> as TypeChecking>::is_indexable(&var_type) {
            Err(SemanticError::ExpectedIndexable)
        } else {
            let Some(fields) = <Either<UserType, StaticType> as GetSubTypes>::get_fields(&var_type)
            else {
                return Err(SemanticError::InvalidPattern);
            };
            if self.index >= fields.len() {
                Err(SemanticError::InvalidPattern)
            } else {
                {
                    let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();
                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ListAccess<Scope> {
    type Output = ();

    type Context = Option<Either<UserType, StaticType>>;

    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self.var.type_of(&scope.borrow())?;

        if !<Either<UserType, StaticType> as TypeChecking>::is_channel(&var_type)
            && !<Either<UserType, StaticType> as TypeChecking>::is_map(&var_type)
            && <Either<UserType, StaticType> as TypeChecking>::is_iterable(&var_type)
        {
            let key_type = match context {
                Some(context) => <Either<UserType, StaticType> as GetSubTypes>::get_key(context),
                None => None,
            };
            let _ = self.index.resolve(scope, &key_type, extra)?;
            let index_type = self.index.type_of(&scope.borrow())?;
            if <Either<UserType, StaticType> as TypeChecking>::is_u64(&index_type) {
                {
                    let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
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
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
                Ok(())
            }
            None => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Slice<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Slice::String { metadata, .. } => {
                match context {
                    Some(context_type) => {
                        let _ = context_type.compatible_with(self, &scope.borrow())?;
                    }
                    None => {}
                }
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            Slice::List { value, metadata } => {
                let (param_context, maybe_length) = match context {
                    Some(context) => (
                        <Either<UserType, StaticType> as GetSubTypes>::get_item(context),
                        <Either<UserType, StaticType> as GetSubTypes>::get_length(context),
                    ),
                    None => (None, None),
                };
                match maybe_length {
                    Some(length) => {
                        if length != value.len() {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                    }
                    None => {}
                }
                for expr in value {
                    let _ = expr.resolve(scope, &param_context, &())?;
                }
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Vector<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Vector::Init {
                value,
                metadata,
                length: _,
                capacity: _,
            } => {
                let param_context = match context {
                    Some(context) => {
                        <Either<UserType, StaticType> as GetSubTypes>::get_item(context)
                    }
                    None => None,
                };

                for expr in value {
                    let _ = expr.resolve(scope, &param_context, &())?;
                }
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            Vector::Def {
                capacity: _,
                metadata,
            } => {
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Tuple<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MultiData<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let maybe_length = match context {
            Some(context) => <Either<UserType, StaticType> as GetSubTypes>::get_length(context),
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
                Some(context) => {
                    <Either<UserType, StaticType> as GetSubTypes>::get_nth(context, &index)
                }
                None => None,
            };
            let _ = expr.resolve(scope, &param_context, &())?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let Some(context) = context else {
            return Err(SemanticError::CantInferType);
        };
        for (index, expr) in self.params.iter().enumerate() {
            let param_context =
                <Either<UserType, StaticType> as GetSubTypes>::get_nth(context, &index);
            let _ = expr.resolve(scope, &param_context, &())?;
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
                            ClosureParam::Full { id, .. } => id,
                            ClosureParam::Minimal(id) => id,
                        },
                        p,
                    )
                })
            })
            .map(|(index, id, param)| {
                let _param_type = param.type_of(&scope.borrow());
                let param_type =
                    <Either<UserType, StaticType> as GetSubTypes>::get_nth(context, &index);
                <Var as BuildVar<Scope>>::build_var(id, &param_type.unwrap())
            })
            .collect::<Vec<Var>>();

        let Some(context_return) =
            <Either<UserType, StaticType> as GetSubTypes>::get_return(context)
        else {
            return Err(SemanticError::CantInferType);
        };

        let _ = self.scope.resolve(scope, &Some(context_return), &vars)?;

        let env_vars = self.scope.find_outer_vars()?;
        {
            let mut borrowed_env = self.env.borrow_mut();
            borrowed_env.extend(env_vars);
        }
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: Some(context.clone()),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ExprScope<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = Vec<Var>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ClosureParam::Full { id: _, signature } => signature.resolve(scope, &(), &()),
            ClosureParam::Minimal(_value) => match context {
                Some(_) => Ok(()),
                None => Err(SemanticError::CantInferType),
            },
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Address<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PtrAccess<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.value.resolve(scope, context, extra)?;
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Channel<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Channel::Receive { addr, metadata, .. } => {
                let _ = addr.resolve(scope, context, extra)?;
                let addr_type = addr.type_of(&scope.borrow())?;
                if !<Either<UserType, StaticType> as TypeChecking>::is_channel(&addr_type) {
                    return Err(SemanticError::ExpectedChannel);
                }
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            Channel::Send {
                addr,
                msg,
                metadata,
            } => {
                let _ = addr.resolve(scope, context, extra)?;
                let addr_type = addr.type_of(&scope.borrow())?;
                if !<Either<UserType, StaticType> as TypeChecking>::is_channel(&addr_type) {
                    return Err(SemanticError::ExpectedChannel);
                }
                let _ = msg.resolve(scope, context, extra)?;
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            Channel::Init {
                value: id,
                metadata,
            } => {
                let _ = scope.borrow_mut().register_chan(id)?;
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Struct<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
            let field_context =
                <Either<UserType, StaticType> as GetSubTypes>::get_field(&user_type, &field_name);

            let _ = expr.resolve(scope, &field_context, &())?;
        }

        let Some(fields_type) =
            <Either<UserType, StaticType> as GetSubTypes>::get_fields(&user_type)
        else {
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
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Union<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
            let field_context = <Either<UserType, StaticType> as GetSubTypes>::get_field(
                &variant_type,
                &field_name,
            );

            let _ = expr.resolve(scope, &field_context, &())?;
        }

        let Some(fields_type) =
            <Either<UserType, StaticType> as GetSubTypes>::get_fields(&variant_type)
        else {
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
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Enum {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
        {
            let mut borrowed_metadata = self.metadata.info.as_ref().borrow_mut();

            *borrowed_metadata = Info::Resolved {
                context: context.clone(),
                signature: Some(self.type_of(&scope.borrow())?),
            };
        }
        Ok(())
        // user_type.compatible_with(&(&self.typename, &self.value), scope)?;
        // Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Map<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Map::Init { fields, metadata } => {
                let item_type = match context {
                    Some(context) => {
                        <Either<UserType, StaticType> as GetSubTypes>::get_item(context)
                    }
                    None => None,
                };

                let key_type = match context {
                    Some(context) => {
                        <Either<UserType, StaticType> as GetSubTypes>::get_key(context)
                    }
                    None => None,
                };
                for (key, value) in fields {
                    let _ = key.resolve(scope, &key_type, &())?;
                    let _ = value.resolve(scope, &item_type, &())?;
                }
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            Map::Def {
                length: _,
                capacity: _,
                metadata,
            } => {
                {
                    let mut borrowed_metadata = metadata.info.as_ref().borrow_mut();

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for KeyData<Scope> {
    type Output = ();
    type Context = Option<Either<UserType, StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
            KeyData::Slice(value) => value.resolve(scope, context, extra),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{
                AddrType, ChanType, KeyType, MapType, NumberType, PrimitiveType, SliceType,
                StaticType, TupleType, VecType,
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
        assert!(res.is_ok());

        let res = primitive.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
            &(),
        );
        assert!(res.is_ok());
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
    fn valid_slice() {
        let string = Slice::parse(r##""Hello World""##.into()).unwrap().1;

        let scope = Scope::new();
        let res = string.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = string.resolve(
            &scope,
            &Some(Either::Static(StaticType::Slice(SliceType::String).into())),
            &(),
        );
        assert!(res.is_ok());

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let scope = Scope::new();
        let res = slice.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Slice(SliceType::List(
                    2,
                    Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    )),
                ))
                .into(),
            )),
            &(),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_slice() {
        let string = Slice::parse(r##""Hello World""##.into()).unwrap().1;
        let scope = Scope::new();

        let res = string.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &(),
        );
        assert!(res.is_err());

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Slice(SliceType::List(
                    2,
                    Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                ))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let res = slice.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Slice(SliceType::List(
                    4,
                    Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    )),
                ))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_vector() {
        let vector = Vector::parse("vec(8)".into()).unwrap().1;

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let vector = Vector::parse("vec[1,2,3]".into()).unwrap().1;

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = vector.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Vec(VecType(Box::new(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ))))
                .into(),
            )),
            &(),
        );
        assert!(res.is_ok());
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
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
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
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ))))
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }
    #[test]
    fn valid_variable_array_complex() {
        let variable = Variable::parse("x[10 + 10]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ))))
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }
    #[test]
    fn robustness_variable_array() {
        let variable = Variable::parse("x[\"Test\"]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::Slice(SliceType::String),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        )),
                    })
                    .into(),
                ),
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
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Tuple(TupleType(vec![
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        ),
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        ),
                    ]))
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
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
                captured: RefCell::new(false),
                address: None,
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
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

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
                captured: RefCell::new(false),
                address: None,
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .unwrap();
        let res = address.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let address_type = address.type_of(&scope.borrow());
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            Either::Static(
                StaticType::Address(AddrType(Box::new(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
                ))))
                .into()
            ),
            address_type
        )
    }

    #[test]
    fn valid_channel() {
        let channel = Channel::parse("receive[&chan1](10)".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                address: None,
                id: "chan1".into(),
                type_sig: Either::Static(
                    StaticType::Chan(ChanType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ))))
                    .into(),
                ),
            })
            .unwrap();

        let res = channel.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let channel = Channel::parse("send[&chan1](10)".into()).unwrap().1;
        let res = channel.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_channel() {
        let channel = Channel::parse("receive[&chan1](10)".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                address: None,
                id: "chan1".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .unwrap();

        let res = channel.resolve(&scope, &None, &());
        assert!(res.is_err());

        let channel = Channel::parse("send[&chan1](10)".into()).unwrap().1;
        let res = channel.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_tuple() {
        let tuple = Tuple::parse("(1,'a')".into()).unwrap().1;
        let scope = Scope::new();
        let res = tuple.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = tuple.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Tuple(TupleType(vec![
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                    Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                ]))
                .into(),
            )),
            &(),
        );
        assert!(res.is_ok());
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
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
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
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
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
        assert!(res.is_ok());

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::Slice(SliceType::String),
                    values_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    )),
                })
                .into(),
            )),
            &(),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_map() {
        let map = Map::parse(r##"map{"x":2,"y":6}"##.into()).unwrap().1;
        let scope = Scope::new();

        let res = map.resolve(
            &scope,
            &Some(Either::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::Slice(SliceType::String),
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
                    keys_type: KeyType::Primitive(PrimitiveType::Number(NumberType::U64)),
                    values_type: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
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
                                StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                    .into(),
                            ),
                        ));
                        res.push((
                            "y".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                    .into(),
                            ),
                        ));
                        res
                    },
                }),
            )
            .unwrap();

        let res = object.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
                                StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
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
        assert!(res.is_ok());
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
        assert!(res.is_ok());
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
