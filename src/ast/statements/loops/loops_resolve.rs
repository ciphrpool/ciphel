use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};
use crate::semantic::{BuildVar, EitherType};
use crate::semantic::{Resolve, RetrieveTypeInfo, ScopeApi, SemanticError, TypeOf};

impl<Scope: ScopeApi> Resolve<Scope> for Loop {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
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
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ForIterator::Id(value) => {
                let var = scope.find_var(value)?;
                // check that the variable is iterable
                if !var.is_iterable() {
                    return Err(SemanticError::ExpectIterable);
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
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ForItem::Id(value) => Ok(()),
            ForItem::Pattern(_) => todo!(),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ForLoop {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.iterator.resolve(scope, context)?;
        let item_type = self.iterator.type_of(scope)?;
        let item_type = <Option<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        > as RetrieveTypeInfo<Scope>>::get_item(&item_type)
        .unwrap();

        let _ = self.item.resolve(scope, context)?;
        // attach the item to the scope
        let mut inner_scope = scope.child_scope()?;

        let items: Vec<Scope::Var> = match &self.item {
            ForItem::Id(id) => vec![Scope::Var::build_from(id, item_type)],
            ForItem::Pattern(pattern) => todo!(),
        };
        inner_scope.attach(items.into_iter());

        let _ = self.scope.resolve(&inner_scope, &None)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for WhileLoop {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope, &None)?;
        // TODO check that the condition is a boolean
        let _ = self.scope.resolve(scope, &None)?;
        Ok(())
    }
}
