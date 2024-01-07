use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::ScopeApi, CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for ExprFlow<Scope> {
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
            ExprFlow::If(value) => value.resolve(scope, context, extra),
            ExprFlow::Match(value) => value.resolve(scope, context, extra),
            ExprFlow::Try(value) => value.resolve(scope, context, extra),
            ExprFlow::Call(value) => value.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfExpr<Scope> {
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
        let _ = self.condition.resolve(scope, context, extra)?;
        // Check if condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }
        let _ = self.main_branch.resolve(scope, context, extra)?;
        let _ = self.else_branch.resolve(scope, context, extra)?;

        let main_branch_type = self.main_branch.type_of(&scope.borrow())?;
        let _ = main_branch_type.compatible_with(&self.else_branch, &scope.borrow())?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Pattern {
    type Output = Vec<Scope::Var>;
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
            Pattern::Primitive(value) => {
                let _ = value.resolve(scope, context, extra)?;
                Ok(Vec::default())
            }
            Pattern::String(value) => {
                let _ = context.compatible_with(value, &scope.borrow())?;
                Ok(Vec::default())
            }
            Pattern::Enum { typename, value } => {
                let borrowed_scope = scope.borrow();
                let user_type: <Scope as ScopeApi>::UserType =
                    borrowed_scope.find_type(typename)?;
                let Some(_) = user_type.get_variant(value) else {
                    return Err(SemanticError::IncorrectVariant);
                };
                Ok(Vec::default())
            }
            Pattern::Union {
                typename,
                variant,
                vars,
            } => {
                let borrowed_scope = scope.borrow();
                let user_type: <Scope as ScopeApi>::UserType =
                    borrowed_scope.find_type(typename)?;
                let variant_type: Option<EitherType<Scope::UserType, Scope::StaticType>> =
                    user_type.get_variant(variant);
                let Some(variant_type) = variant_type else {
                    return Err(SemanticError::CantInferType);
                };
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(
                    &variant_type
                ) else {
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
            Pattern::Struct { typename, vars } => {
                let borrowed_scope = scope.borrow();
                let user_type: <Scope as ScopeApi>::UserType =
                    borrowed_scope.find_type(typename)?;
                let user_type = user_type.type_of(&scope.borrow())?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
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
                let Some(use_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(&use_type) else {
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
impl<Scope: ScopeApi> Resolve<Scope> for PatternExpr<Scope> {
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
        let vars = self.pattern.resolve(scope, context, extra)?;
        // create a scope and assign the pattern variable to it before resolving the expression
        let _ = self.expr.resolve(scope, context, &())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchExpr<Scope> {
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
        let _ = self.expr.resolve(scope, context, extra)?;
        let expr_type = Some(self.expr.type_of(&scope.borrow())?);

        let _ = self.else_branch.resolve(scope, &expr_type, &())?;

        for pattern in &self.patterns {
            let _ = pattern.resolve(scope, &expr_type, &())?;
        }
        let else_branch_type = self.else_branch.type_of(&scope.borrow())?;

        let (maybe_err, _) =
            self.patterns
                .iter()
                .fold((None, else_branch_type), |mut previous, pattern| {
                    if let Err(e) = previous.1.compatible_with(pattern, &scope.borrow()) {
                        previous.0 = Some(e);
                        previous
                    } else {
                        match pattern.type_of(&scope.borrow()) {
                            Ok(pattern_type) => (None, pattern_type),
                            Err(err) => (
                                Some(err),
                                EitherType::Static(Scope::StaticType::build_unit()),
                            ),
                        }
                    }
                });
        if let Some(err) = maybe_err {
            return Err(err);
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryExpr<Scope> {
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
        let _ = self.try_branch.resolve(scope, context, extra)?;
        let _ = self.else_branch.resolve(scope, context, extra)?;

        let try_branch_type = self.try_branch.type_of(&scope.borrow())?;
        let _ = try_branch_type.compatible_with(&self.else_branch, &scope.borrow())?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FnCall<Scope> {
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
        let _ = self.fn_var.resolve(scope, context, extra)?;
        let fn_var_type = self.fn_var.type_of(&scope.borrow())?;
        if !<EitherType<<Scope as ScopeApi>::UserType,
             <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>
             ::is_callable(&fn_var_type) {
            return Err(SemanticError::ExpectedCallable);
        }

        for (index, expr) in self.params.iter().enumerate() {
            let param_context = <EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            > as GetSubTypes<Scope>>::get_nth(&fn_var_type, &index);
            let _ = expr.resolve(scope, &param_context, &())?;
        }

        let Some(fields) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_fields(&fn_var_type) else {
            return Err(SemanticError::ExpectedCallable);
        };
        if self.params.len() != fields.len() {
            return Err(SemanticError::IncorrectStruct);
        }
        for (index, (_, field_type)) in fields.iter().enumerate() {
            let _ = field_type.compatible_with(&self.params[index], &scope.borrow())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::TryParse, semantic::scope::scope_impl::Scope};

    use super::*;

    #[test]
    fn valid_if() {
        let expr = IfExpr::parse("if true {10} else {20}".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_if() {
        let expr = IfExpr::parse("if 10 {10} else {20}".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = IfExpr::parse("if true {10} else {'a'}".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}
