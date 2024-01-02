use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> Resolve<Scope> for ExprFlow {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ExprFlow::If(value) => value.resolve(scope, context),
            ExprFlow::Match(value) => value.resolve(scope, context),
            ExprFlow::Try(value) => value.resolve(scope, context),
            ExprFlow::Call(value) => value.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfExpr {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope, context)?;
        // Check if condition is a boolean
        let condition_type = self.condition.type_of(scope)?;
        if !<Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }
        let _ = self.main_branch.resolve(scope, context)?;
        let _ = self.else_branch.resolve(scope, context)?;

        let main_branch_type = self.main_branch.type_of(scope)?;
        let _ = main_branch_type.compatible_with(&self.else_branch, scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Pattern {
    type Output = Vec<Scope::Var>;
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Pattern::Primitive(value) => {
                let _ = value.resolve(scope, context)?;
                Ok(Vec::default())
            }
            Pattern::String(value) => {
                let _ = context.compatible_with(value, scope)?;
                Ok(Vec::default())
            }
            Pattern::Enum { typename, value } => {
                let user_type = scope.find_type(typename)?;
                let variant_type = user_type.get_variant(value);
                match variant_type {
                    Some(variant_type) => {
                        if <EitherType<
                            <Scope as ScopeApi>::UserType,
                            <Scope as ScopeApi>::StaticType,
                        > as TypeChecking<Scope>>::is_enum_variant(
                            &variant_type
                        ) {
                            Ok(Vec::default())
                        } else {
                            Err(SemanticError::IncorrectVariant)
                        }
                    }
                    None => Err(SemanticError::IncorrectVariant),
                }
            }
            Pattern::UnionInline {
                typename,
                variant,
                vars,
            } => {
                let user_type: &<Scope as ScopeApi>::UserType = scope.find_type(typename)?;
                let variant_type: Option<EitherType<Scope::UserType, Scope::StaticType>> =
                    user_type.get_variant(variant);
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) =
                    <Option<
                        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                    > as GetSubTypes<Scope>>::get_fields(&variant_type)
                else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &vars[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            Pattern::UnionFields {
                typename,
                variant,
                vars,
            } => {
                let user_type: &<Scope as ScopeApi>::UserType = scope.find_type(typename)?;
                let variant_type: Option<EitherType<Scope::UserType, Scope::StaticType>> =
                    user_type.get_variant(variant);
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) =
                    <Option<
                        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                    > as GetSubTypes<Scope>>::get_fields(&variant_type)
                else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (field_name, field_type) in fields.iter() {
                    let Some(var_name) = vars.iter().find(|name| {
                        field_name
                            .clone()
                            .map(|inner| if inner == **name { Some(()) } else { None })
                            .flatten()
                            .is_some()
                    }) else {
                        return Err(SemanticError::InvalidPattern);
                    };
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            Pattern::StructInline { typename, vars } => {
                let user_type: &<Scope as ScopeApi>::UserType = scope.find_type(typename)?;
                let user_type = user_type.type_of(scope)?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as GetSubTypes<Scope>>::get_fields(&user_type) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &vars[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            Pattern::StructFields { typename, vars } => {
                let user_type: &<Scope as ScopeApi>::UserType = scope.find_type(typename)?;
                let user_type = user_type.type_of(scope)?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as GetSubTypes<Scope>>::get_fields(&user_type) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (field_name, field_type) in fields.iter() {
                    let Some(var_name) = vars.iter().find(|name| {
                        field_name
                            .clone()
                            .map(|inner| if inner == **name { Some(()) } else { None })
                            .flatten()
                            .is_some()
                    }) else {
                        return Err(SemanticError::InvalidPattern);
                    };
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            Pattern::Tuple(value) => {
                let mut scope_vars = Vec::with_capacity(value.len());
                let Some(fields) = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as GetSubTypes<Scope>>::get_fields(&context) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if value.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &value[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternExpr {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let vars = self.pattern.resolve(scope, context)?;
        // create a scope and assign the pattern variable to it before resolving the expression
        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(vars.into_iter());
        let _ = self.expr.resolve(&mut inner_scope, context)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchExpr {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.expr.resolve(scope, context)?;
        let expr_type = self.expr.type_of(scope)?;

        for pattern in &self.patterns {
            let _ = pattern.resolve(scope, &expr_type)?;
        }

        let _ = self.else_branch.resolve(scope, &expr_type)?;

        let else_branch_type = self.else_branch.type_of(scope)?;

        let (maybe_err, _) =
            self.patterns
                .iter()
                .fold((None, else_branch_type), |mut previous, pattern| {
                    if let Err(e) = previous.1.compatible_with(pattern, scope) {
                        previous.0 = Some(e);
                        previous
                    } else {
                        match pattern.type_of(scope) {
                            Ok(pattern_type) => (None, pattern_type),
                            Err(err) => (Some(err), None),
                        }
                    }
                });
        if let Some(err) = maybe_err {
            return Err(err);
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryExpr {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.try_branch.resolve(scope, context)?;
        let _ = self.else_branch.resolve(scope, context)?;

        let try_branch_type = self.try_branch.type_of(scope)?;
        let _ = try_branch_type.compatible_with(&self.else_branch, scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FnCall {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.fn_var.resolve(scope, context)?;
        let fn_var_type = self.fn_var.type_of(scope)?;
        if !<Option<EitherType<<Scope as ScopeApi>::UserType,
             <Scope as ScopeApi>::StaticType>> as TypeChecking<Scope>>
             ::is_callable(&fn_var_type) {
            return Err(SemanticError::ExpectedCallable);
        }

        for (index, expr) in self.params.iter().enumerate() {
            let param_context = <Option<
                EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
            > as GetSubTypes<Scope>>::get_nth(&fn_var_type, &index);
            let _ = expr.resolve(scope, &param_context)?;
        }

        let Some(fields) = <Option<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        > as GetSubTypes<Scope>>::get_fields(&fn_var_type) else {
            return Err(SemanticError::ExpectedCallable);
        };
        if self.params.len() != fields.len() {
            return Err(SemanticError::IncorrectStruct);
        }
        for (index, (_, field_type)) in fields.iter().enumerate() {
            let _ = field_type.compatible_with(&self.params[index], scope)?;
        }

        Ok(())
    }
}
