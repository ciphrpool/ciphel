use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> Resolve<Scope> for Loop {
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
            Loop::For(value) => value.resolve(scope, context),
            Loop::While(value) => value.resolve(scope, context),
            Loop::Loop(value) => value.resolve(scope, &None),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForIterator {
    type Output = ();
    type Context = ();
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
            ForIterator::Id(value) => {
                let var = scope.find_var(value)?;
                // check that the variable is iterable
                let var_type = var.type_of(scope)?;
                if !<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
                 as TypeChecking<Scope>>::is_iterable(&var_type)
                {
                    return Err(SemanticError::ExpectedIterable);
                }
                Ok(())
            }
            ForIterator::Vec(value) => value.resolve(scope, &None),
            ForIterator::Slice(value) => value.resolve(scope, &None),
            ForIterator::Tuple(value) => value.resolve(scope, &None),
            ForIterator::Receive { addr, .. } => addr.resolve(scope, &None),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ForItem {
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
            ForItem::Id(value) => {
                let Some(item_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                Ok(vec![Scope::Var::build_var(value, &item_type)])
            }
            ForItem::Pattern(pattern) => pattern.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForLoop {
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
        let _ = self.iterator.resolve(scope, &())?;
        let item_type = self.iterator.type_of(scope)?;
        let item_type = <
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
        as GetSubTypes<Scope>>::get_item(&item_type);

        let item_vars = self.item.resolve(scope, &item_type)?;
        // attach the item to the scope
        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(item_vars.into_iter());

        let _ = self.scope.resolve(&mut inner_scope, context)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for WhileLoop {
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
        let _ = self.condition.resolve(scope, &None)?;
        // check that the condition is a boolean
        let condition_type = self.condition.type_of(scope)?;
        if !<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }
        let _ = self.scope.resolve(scope, context)?;
        Ok(())
    }
}
