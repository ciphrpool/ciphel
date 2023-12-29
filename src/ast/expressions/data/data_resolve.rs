use super::{
    Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, FieldAccess, KeyData,
    ListAccess, Map, MultiData, Primitive, PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable,
    Vector,
};
use crate::semantic::BuildVar;
use crate::semantic::{
    CompatibleWith, EitherType, Resolve, RetrieveTypeInfo, ScopeApi, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> Resolve<Scope> for Data {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope, context),
            Data::Slice(value) => value.resolve(scope, context),
            Data::Vec(value) => value.resolve(scope, context),
            Data::Closure(value) => value.resolve(scope, context),
            Data::Chan(value) => value.resolve(scope, context),
            Data::Tuple(value) => value.resolve(scope, context),
            Data::Address(value) => value.resolve(scope, context),
            Data::PtrAccess(value) => value.resolve(scope, context),
            Data::Variable(value) => value.resolve(scope, context),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope, context),
            Data::Struct(value) => value.resolve(scope, context),
            Data::Union(value) => value.resolve(scope, context),
            Data::Enum(value) => value.resolve(scope, context),
        }
    }
}

impl Variable {
    fn resolve_based<Scope>(
        &self,
        context: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(VarID(value)) => <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as RetrieveTypeInfo<Scope>>::get_field(
                context, value
            )
            .ok_or(SemanticError::UnknownField),
            Variable::FieldAccess(FieldAccess { var, field }) => {
                let var_type = var.resolve_based::<Scope>(context)?;
                field.resolve_based::<Scope>(&var_type)
            }
            Variable::ListAccess(ListAccess { var, .. }) => {
                let var_type = var.resolve_based::<Scope>(context)?;
                if !<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as RetrieveTypeInfo<Scope>>::is_iterable(&var_type) {
                    Err(SemanticError::ExpectIterable)
                } else {
                    Ok(var_type)
                }
            }
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Variable {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(VarID(value)) => {
                let _ = scope.find_var(value)?;

                Ok(())
            }
            Variable::FieldAccess(FieldAccess { var, field }) => {
                let _ = var.resolve(scope, context)?;
                let var_type = var.type_of(scope)?;
                let Some(var_type) = var_type else {
                    return Err(SemanticError::CantInferType);
                };
                let _ = field.resolve_based::<Scope>(&var_type)?;
                Ok(())
            }
            Variable::ListAccess(ListAccess { var, .. }) => {
                let _ = var.resolve(scope, context)?;
                let var_type = var.type_of(scope)?;
                if !<Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as RetrieveTypeInfo<Scope>>::is_iterable(&var_type)
                {
                    Err(SemanticError::ExpectIterable)
                } else {
                    Ok(())
                }
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Primitive {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match context {
            Some(context_type) => {
                let _ = context_type.compatible_with(self, scope)?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Slice {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Slice::String(value) => Ok(()),
            Slice::List(value) => match value.iter().find_map(|expr| {
                let param_context = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as RetrieveTypeInfo<Scope>>::get_item(context);

                expr.resolve(scope, &param_context).err()
            }) {
                Some(err) => Err(err),
                None => Ok(()),
            },
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Vector {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Vector::Init(value) => match value.iter().find_map(|expr| {
                let param_context = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as RetrieveTypeInfo<Scope>>::get_item(context);

                expr.resolve(scope, &param_context).err()
            }) {
                Some(err) => Err(err),
                None => Ok(()),
            },
            Vector::Def { length, capacity } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Tuple {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope, context)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MultiData {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self.iter().enumerate().find_map(|(index, expr)| {
            let param_context = <Option<
                EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
            > as RetrieveTypeInfo<Scope>>::get_nth(context, &index);

            expr.resolve(scope, &param_context).err()
        }) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = {
            match self.params.iter().enumerate().find_map(|(index, expr)| {
                let param_context =
                    <Option<
                        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                    > as RetrieveTypeInfo<Scope>>::get_nth(context, &index);

                expr.resolve(scope, &param_context).err()
            }) {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;

        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(self.params.iter().enumerate().map(|(index, param)| {
            let param_type = param.type_of(scope).unwrap_or(None);
            let param_type = match param_type {
                Some(param_type) => Some(param_type),
                None => <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as RetrieveTypeInfo<Scope>>::get_nth(context, &index),
            };
            let id = match param {
                ClosureParam::Full { id, .. } => id,
                ClosureParam::Minimal(id) => id,
            };
            Scope::Var::build_var(id, &param_type.unwrap())
        }));
        let _ = self.scope.resolve(&inner_scope, context)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureScope {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ClosureScope::Scope(value) => value.resolve(scope, context),
            ClosureScope::Expr(value) => value.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureParam {
    type Output = ();
    type Context =
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ClosureParam::Full { id, signature } => signature.resolve(scope, &()),
            ClosureParam::Minimal(value) => match context {
                Some(_) => Ok(()),
                None => Err(SemanticError::CantInferType),
            },
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Address {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope, context)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PtrAccess {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope, context)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Channel {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Channel::Receive { addr, .. } => addr.resolve(scope, context),
            Channel::Send { addr, msg } => {
                let _ = addr.resolve(scope, context)?;
                let _ = msg.resolve(scope, context)?;
                Ok(())
            }
            Channel::Init(id) => scope.register_chan(id),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Struct {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Struct::Inline { id, data } => {
                let user_type = scope.find_type(id)?;
                let user_type = user_type.type_of(scope)?;
                let _ = data.resolve(scope, &user_type)?;
                let _ = user_type.compatible_with(data, scope)?;
                Ok(())
            }
            Struct::Field { id, fields } => {
                let user_type = scope.find_type(id)?;
                let user_type = user_type.type_of(scope)?;
                let _ = {
                    match fields.iter().find_map(|(field_name, expr)| {
                        let field_context = <Option<
                            EitherType<
                                <Scope as ScopeApi>::UserType,
                                <Scope as ScopeApi>::StaticType,
                            >,
                        > as RetrieveTypeInfo<Scope>>::get_field(
                            context, &field_name
                        );

                        expr.resolve(scope, &field_context).err()
                    }) {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                }?;
                let _ = user_type.compatible_with(fields, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Union {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Union::Inline {
                typename,
                variant,
                data,
            } => {
                let user_type = scope.find_type(typename)?;
                let variant_type = user_type.get_variant(variant);
                let _ = data.resolve(scope, &variant_type)?;

                let _ = user_type.compatible_with(&(variant, data), scope)?;
                Ok(())
            }
            Union::Field {
                typename,
                variant,
                fields,
            } => {
                let user_type = scope.find_type(typename)?;
                let variant_type = user_type.get_variant(variant);
                let _ = {
                    match fields.iter().find_map(|(field_name, expr)| {
                        let field_context = <Option<
                            EitherType<
                                <Scope as ScopeApi>::UserType,
                                <Scope as ScopeApi>::StaticType,
                            >,
                        > as RetrieveTypeInfo<Scope>>::get_field(
                            &variant_type, &field_name
                        );

                        expr.resolve(scope, &field_context).err()
                    }) {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                }?;
                let _ = user_type.compatible_with(&(variant, fields), scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Enum {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let user_type = scope.find_type(&self.typename)?;
        let variant_type = user_type.get_variant(&self.value);
        match variant_type {
            Some(variant_type) => {
                if <EitherType<<Scope as ScopeApi>::UserType, 
                    <Scope as ScopeApi>::StaticType> as RetrieveTypeInfo<Scope>>
                    ::is_enum_variant(&variant_type) {
                    Ok(())
                } else {
                    Err(SemanticError::IncorrectVariant)
                }
            }
            None => Err(SemanticError::IncorrectVariant),
        }
        // user_type.compatible_with(&(&self.typename, &self.value), scope)?;
        // Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Map {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {

        match self {
            Map::Init { fields } => {
                let item_type = <Option<EitherType<<Scope as ScopeApi>::UserType,
                 <Scope as ScopeApi>::StaticType>> as RetrieveTypeInfo<Scope>>::get_item(context);

                match fields
                    .iter()
                    .find_map(|(key, value)| match key.resolve(scope, context) {
                        Ok(_) => value.resolve(scope, &item_type).err(),
                        Err(e) => Some(e),
                    }) {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            }
            Map::Def { length, capacity } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for KeyData {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            KeyData::Number(_) => Ok(()),
            KeyData::Bool(_) => Ok(()),
            KeyData::Char(_) => Ok(()),
            KeyData::String(_) => Ok(()),
            KeyData::Address(value) => value.resolve(scope, context),
            KeyData::Enum(value) => value.resolve(scope, context),
        }
    }
}
