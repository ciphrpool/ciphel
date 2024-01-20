use super::{
    Address, Channel, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData,
    ListAccess, Map, MultiData, NumAccess, Primitive, PtrAccess, Slice, Struct, Tuple, Union,
    VarID, Variable, Vector,
};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Data<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
        context: &EitherType<InnerScope::UserType, InnerScope::StaticType>,
    ) -> Result<EitherType<InnerScope::UserType, InnerScope::StaticType>, SemanticError>
    where
        Self: Sized,
        InnerScope: ScopeApi,
    {
        match self {
            Variable::Var(VarID(value)) => <EitherType<
                <InnerScope as ScopeApi>::UserType,
                <InnerScope as ScopeApi>::StaticType,
            > as GetSubTypes<InnerScope>>::get_field(
                context, value
            )
            .ok_or(SemanticError::UnknownField),
            Variable::FieldAccess(FieldAccess { var, field }) => {
                let var_type = var.resolve_based(scope, context)?;
                field.resolve_based(scope, &var_type)
            }
            Variable::ListAccess(ListAccess { var, index }) => {
                let var_type = var.resolve_based(scope, context)?;
                if !<EitherType<
                    <InnerScope as ScopeApi>::UserType,
                    <InnerScope as ScopeApi>::StaticType,
                > as TypeChecking<InnerScope>>::is_iterable(&var_type)
                {
                    Err(SemanticError::ExpectedIterable)
                } else {
                    let key_type =
                        <EitherType<
                            <InnerScope as ScopeApi>::UserType,
                            <InnerScope as ScopeApi>::StaticType,
                        > as GetSubTypes<InnerScope>>::get_key(&var_type);

                    let _ = index.resolve(scope, &key_type, &())?;
                    let index_type = index.type_of(&scope.borrow())?;
                    let _ = key_type.compatible_with(&index_type, &scope.borrow())?;
                    Ok(var_type)
                }
            }
            Variable::NumAccess(_) => todo!(),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Variable<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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

    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;

    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = scope.borrow().find_var(&self.0)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FieldAccess<Scope> {
    type Output = ();

    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;

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
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for NumAccess<Scope> {
    type Output = ();

    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;

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
        if !<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
        as TypeChecking<Scope>>::is_indexable(&var_type)
        {
            Err(SemanticError::ExpectedIndexable)
        } else {
            Ok(())
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ListAccess<Scope> {
    type Output = ();

    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;

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
        if !<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
            as TypeChecking<Scope>>::is_channel(&var_type)
        && !<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
            as TypeChecking<Scope>>::is_iterable(&var_type)
        {
            Err(SemanticError::ExpectedIndexable)
        } else {

            let key_type = match context {
                Some(context) => <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_key(context),
                None => None,
            };
            let _ = self.index.resolve(scope, &key_type, extra)?;
            let index_type = self.index.type_of(&scope.borrow())?;
            let _ = key_type.compatible_with(&index_type, &scope.borrow())?;
            Ok(())
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Primitive {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            Slice::String(..) => match context {
                Some(context_type) => {
                    let _ = context_type.compatible_with(self, &scope.borrow())?;
                    Ok(())
                }
                None => Ok(()),
            },
            Slice::List(value) => {
                let (param_context, maybe_length) =
                    match context {
                        Some(context) => (
                            <EitherType<
                                <Scope as ScopeApi>::UserType,
                                <Scope as ScopeApi>::StaticType,
                            > as GetSubTypes<Scope>>::get_item(context),
                            <EitherType<
                                <Scope as ScopeApi>::UserType,
                                <Scope as ScopeApi>::StaticType,
                            > as GetSubTypes<Scope>>::get_length(
                                context
                            ),
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
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Vector<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            Vector::Init(value) => {
                let param_context = match context {
                    Some(context) => <EitherType<
                        <Scope as ScopeApi>::UserType,
                        <Scope as ScopeApi>::StaticType,
                    > as GetSubTypes<Scope>>::get_item(context),
                    None => None,
                };

                for expr in value {
                    let _ = expr.resolve(scope, &param_context, &())?;
                }
                Ok(())
            }
            Vector::Def {
                length: _,
                capacity: _,
            } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Tuple<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
        self.0.resolve(scope, context, extra)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MultiData<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            Some(context) => <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as GetSubTypes<Scope>>::get_length(context),
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
                Some(context) => <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_nth(context, &index),
                None => None,
            };
            let _ = expr.resolve(scope, &param_context, &())?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            let param_context = <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as GetSubTypes<Scope>>::get_nth(context, &index);
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
                let param_type = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_nth(context, &index);
                Scope::Var::build_var(id, &param_type.unwrap())
            })
            .collect::<Vec<Scope::Var>>();

        let Some(context_return) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_return(context) else {
            return Err(SemanticError::CantInferType);
        };

        let _ = self.scope.resolve(scope, &Some(context_return), &vars)?;

        let env_vars = self.scope.find_outer_vars()?;
        {
            let mut borrowed_env = self.env.borrow_mut();
            borrowed_env.extend(env_vars);
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ExprScope<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = Vec<Scope::Var>;
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
    type Context =
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>;
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
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
        self.0.resolve(scope, context, extra)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PtrAccess<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
        self.0.resolve(scope, context, extra)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Channel<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            Channel::Receive { addr, .. } => {
                let _ = addr.resolve(scope, context, extra)?;
                let addr_type = addr.type_of(&scope.borrow())?;
                if ! <EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_channel(&addr_type) {
                    return Err(SemanticError::ExpectedChannel);
                }
                Ok(())
            }
            Channel::Send { addr, msg } => {
                let _ = addr.resolve(scope, context, extra)?;
                let addr_type = addr.type_of(&scope.borrow())?;
                if ! <EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_channel(&addr_type) {
                    return Err(SemanticError::ExpectedChannel);
                }
                let _ = msg.resolve(scope, context, extra)?;
                Ok(())
            }
            Channel::Init(id) => scope.borrow_mut().register_chan(id),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Struct<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
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
            let field_context = <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as GetSubTypes<Scope>>::get_field(
                &user_type, &field_name
            );

            let _ = expr.resolve(scope, &field_context, &())?;
        }

        let Some(fields_type) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_fields(&user_type) else {
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
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Union<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
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
            let field_context = <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as GetSubTypes<Scope>>::get_field(
                &variant_type, &field_name
            );

            let _ = expr.resolve(scope, &field_context, &())?;
        }

        let Some(fields_type) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_fields(&variant_type) else {
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
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Enum {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
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
        Ok(())
        // user_type.compatible_with(&(&self.typename, &self.value), scope)?;
        // Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Map<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
            Map::Init { fields } => {
                let item_type = match context {
                    Some(context) => <EitherType<
                        <Scope as ScopeApi>::UserType,
                        <Scope as ScopeApi>::StaticType,
                    > as GetSubTypes<Scope>>::get_item(context),
                    None => None,
                };

                let key_type = match context {
                    Some(context) => <EitherType<
                        <Scope as ScopeApi>::UserType,
                        <Scope as ScopeApi>::StaticType,
                    > as GetSubTypes<Scope>>::get_key(context),
                    None => None,
                };
                for (key, value) in fields {
                    let _ = key.resolve(scope, &key_type, &())?;
                    let _ = value.resolve(scope, &item_type, &())?;
                }

                Ok(())
            }
            Map::Def {
                length: _,
                capacity: _,
            } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for KeyData<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
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
                AddrType, ChanType, KeyType, MapType, PrimitiveType, SliceType, StaticType,
                TupleType, VecType,
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
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
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
            &Some(EitherType::Static(
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
            &Some(EitherType::Static(
                StaticType::Slice(SliceType::String).into(),
            )),
            &(),
        );
        assert!(res.is_ok());

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let scope = Scope::new();
        let res = slice.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = slice.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Slice(SliceType::List(
                    2,
                    Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
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
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Bool).into(),
            )),
            &(),
        );
        assert!(res.is_err());

        let slice = Slice::parse("[1,2]".into()).unwrap().1;

        let res = slice.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Slice(SliceType::List(
                    2,
                    Box::new(EitherType::Static(
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
            &Some(EitherType::Static(
                StaticType::Slice(SliceType::List(
                    4,
                    Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
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
        let vector = Vector::parse("vec(2,8)".into()).unwrap().1;

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let vector = Vector::parse("vec[1,2,3]".into()).unwrap().1;

        let scope = Scope::new();
        let res = vector.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let res = vector.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Vec(VecType(Box::new(EitherType::Static(
                    StaticType::Primitive(PrimitiveType::Number).into(),
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
            &Some(EitherType::Static(
                StaticType::Vec(VecType(Box::new(EitherType::Static(
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
                id: "x".into(),
                type_sig: EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable_type = variable.type_of(&scope.borrow());
        assert!(variable_type.is_ok());
        let variable_type = variable_type.unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            variable_type
        );

        let variable = Variable::parse("x[10]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                id: "x".into(),
                type_sig: EitherType::Static(
                    StaticType::Vec(VecType(Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
                    ))))
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable = Variable::parse("x[10 + 10]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                id: "x".into(),
                type_sig: EitherType::Static(
                    StaticType::Vec(VecType(Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
                    ))))
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable = Variable::parse("x[\"Test\"]".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                id: "x".into(),
                type_sig: EitherType::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::Slice(SliceType::String),
                        values_type: Box::new(EitherType::Static(
                            StaticType::Primitive(PrimitiveType::Number).into(),
                        )),
                    })
                    .into(),
                ),
            })
            .unwrap();
        let res = variable.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let variable = Variable::parse("x.0".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                id: "x".into(),
                type_sig: EitherType::Static(
                    StaticType::Tuple(TupleType(vec![
                        EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                        EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
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
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            variable_type
        );

        let variable = Variable::parse("point.x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                id: "point".into(),
                type_sig: EitherType::User(
                    UserType::Struct(
                        user_type_impl::Struct {
                            id: "Point".into(),
                            fields: {
                                let mut res = Vec::new();
                                res.push((
                                    "x".into(),
                                    EitherType::Static(
                                        StaticType::Primitive(PrimitiveType::Number).into(),
                                    ),
                                ));
                                res.push((
                                    "y".into(),
                                    EitherType::Static(
                                        StaticType::Primitive(PrimitiveType::Number).into(),
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
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
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
                id: "x".into(),
                type_sig: EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            })
            .unwrap();
        let res = address.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let address_type = address.type_of(&scope.borrow());
        assert!(address_type.is_ok());
        let address_type = address_type.unwrap();
        assert_eq!(
            EitherType::Static(
                StaticType::Address(AddrType(Box::new(EitherType::Static(
                    StaticType::Primitive(PrimitiveType::Number).into()
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
                id: "chan1".into(),
                type_sig: EitherType::Static(
                    StaticType::Chan(ChanType(Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
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
                id: "chan1".into(),
                type_sig: EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
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
            &Some(EitherType::Static(
                StaticType::Tuple(TupleType(vec![
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
            &Some(EitherType::Static(
                StaticType::Tuple(TupleType(vec![
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                ]))
                .into(),
            )),
            &(),
        );
        assert!(res.is_err());

        let res = tuple.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Tuple(TupleType(vec![
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                    EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
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
            &Some(EitherType::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::Slice(SliceType::String),
                    values_type: Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
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
            &Some(EitherType::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::Slice(SliceType::String),
                    values_type: Box::new(EitherType::Static(
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
            &Some(EitherType::Static(
                StaticType::Map(MapType {
                    keys_type: KeyType::Primitive(PrimitiveType::Number),
                    values_type: Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number).into(),
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
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                        ));
                        res.push((
                            "y".into(),
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
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
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                        ));
                        res.push((
                            "y".into(),
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
                        let mut res = HashMap::new();
                        res.insert(
                            "Point".into(),
                            user_type_impl::Struct {
                                id: "Point".into(),
                                fields: vec![
                                    (
                                        "x".into(),
                                        EitherType::Static(
                                            StaticType::Primitive(PrimitiveType::Number).into(),
                                        ),
                                    ),
                                    (
                                        "y".into(),
                                        EitherType::Static(
                                            StaticType::Primitive(PrimitiveType::Number).into(),
                                        ),
                                    ),
                                ],
                            },
                        );
                        res.insert(
                            "Axe".into(),
                            user_type_impl::Struct {
                                id: "Axe".into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push((
                                        "x".into(),
                                        EitherType::Static(
                                            StaticType::Primitive(PrimitiveType::Number).into(),
                                        ),
                                    ));
                                    res
                                },
                            },
                        );
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
                        let mut res = HashMap::new();
                        res.insert(
                            "Point".into(),
                            user_type_impl::Struct {
                                id: "Point".into(),
                                fields: vec![
                                    (
                                        "x".into(),
                                        EitherType::Static(
                                            StaticType::Primitive(PrimitiveType::Number).into(),
                                        ),
                                    ),
                                    (
                                        "y".into(),
                                        EitherType::Static(
                                            StaticType::Primitive(PrimitiveType::Char).into(),
                                        ),
                                    ),
                                ],
                            },
                        );
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
                        let mut res = HashSet::new();
                        res.insert("Point".into());
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
                        let mut res = HashSet::new();
                        res.insert("Axe".into());
                        res
                    },
                }),
            )
            .unwrap();
        let res = object.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}
