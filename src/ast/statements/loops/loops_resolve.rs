use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Loop<Scope> {
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
            Loop::For(value) => value.resolve(scope, context, extra),
            Loop::While(value) => value.resolve(scope, context, extra),
            Loop::Loop(value) => {
                let _ = value.resolve(scope, &context, &Vec::default())?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForIterator<Scope> {
    type Output = ();
    type Context = ();
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
            ForIterator::Id(value) => {
                let borrowed_scope = scope.borrow();
                let var = borrowed_scope.find_var(value)?;
                // check that the variable is iterable
                let var_type = var.type_of(&scope.borrow())?;
                if !<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
                 as TypeChecking<Scope>>::is_iterable(&var_type)
                {
                    return Err(SemanticError::ExpectedIterable);
                }
                Ok(())
            }
            ForIterator::Vec(value) => value.resolve(scope, &None, &()),
            ForIterator::Slice(value) => value.resolve(scope, &None, &()),
            ForIterator::Tuple(value) => value.resolve(scope, &None, &()),
            ForIterator::Receive { addr, .. } => addr.resolve(scope, &None, &()),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ForItem {
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
            ForItem::Id(value) => {
                let Some(item_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                Ok(vec![Scope::Var::build_var(value, &item_type)])
            }
            ForItem::Pattern(pattern) => pattern.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForLoop<Scope> {
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
        let _ = self.iterator.resolve(scope, &(), &())?;
        let item_type = self.iterator.type_of(&scope.borrow())?;
        let item_type = <
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
        as GetSubTypes<Scope>>::get_item(&item_type);

        let item_vars = self.item.resolve(scope, &item_type, &())?;
        // attach the item to the scope

        let _ = self.scope.resolve(scope, context, &item_vars)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for WhileLoop<Scope> {
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
        let _ = self.condition.resolve(scope, &None, &())?;
        // check that the condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }
        let _ = self.scope.resolve(scope, context, &Vec::default())?;
        Ok(())
    }
}
