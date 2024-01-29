use super::{ForItem, ForIterator, ForLoop, Loop, WhileLoop};
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::Either;
use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > Resolve<Scope> for Loop<Scope>
{
    type Output = ();
    type Context = Option<Either<Scope::UserType, Scope::StaticType>>;
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
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > Resolve<Scope> for ForIterator<Scope>
{
    type Output = ();
    type Context = ();
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
        match self {
            ForIterator::Id(value) => {
                let borrowed_scope = scope.borrow();
                let var = borrowed_scope.find_var(value)?;
                // check that the variable is iterable
                let var_type = var.type_of(&scope.borrow())?;
                if !<
                    Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
                 as TypeChecking<Scope>>::is_iterable(&var_type)
                {
                    return Err(SemanticError::ExpectedIterable);
                }
                Ok(())
            }
            ForIterator::Vec(value) => value.resolve(scope, &None, &()),
            ForIterator::Slice(value) => value.resolve(scope, &None, &()),
            ForIterator::Receive { addr, .. } => addr.resolve(scope, &None, &()),
        }
    }
}

impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > Resolve<Scope> for ForItem
{
    type Output = Vec<Scope::Var>;
    type Context = Option<Either<Scope::UserType, Scope::StaticType>>;
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
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > Resolve<Scope> for ForLoop<Scope>
{
    type Output = ();
    type Context = Option<Either<Scope::UserType, Scope::StaticType>>;
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
        let _ = self.iterator.resolve(scope, &(), &())?;
        let item_type = self.iterator.type_of(&scope.borrow())?;
        let item_type = <
            Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>
        as GetSubTypes<Scope>>::get_item(&item_type);

        let item_vars = self.item.resolve(scope, &item_type, &())?;
        // attach the item to the scope
        let _ = self.scope.resolve(scope, context, &item_vars)?;
        Ok(())
    }
}
impl<
        Scope: ScopeApi<
            UserType = UserType,
            StaticType = StaticType,
            Var = Var,
            Chan = Chan,
            Event = Event,
        >,
    > Resolve<Scope> for WhileLoop<Scope>
{
    type Output = ();
    type Context = Option<Either<Scope::UserType, Scope::StaticType>>;
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
        let _ = self.condition.resolve(scope, &None, &())?;
        // check that the condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<Either<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }
        let _ = self.scope.resolve(scope, context, &Vec::default())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl,
            static_types::{NumberType, PrimitiveType, StaticType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_for_loop() {
        let expr_loop = ForLoop::<scope_impl::Scope>::parse(
            r##"
        for i in [1,2,3] {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr_loop = ForLoop::<scope_impl::Scope>::parse(
            r##"
        for (i,j) in [(1,1),(2,2),(3,3)] {
            x = j;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_for_loop() {
        let expr_loop = ForLoop::<scope_impl::Scope>::parse(
            r##"
        for i in y {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr_loop = ForLoop::<scope_impl::Scope>::parse(
            r##"
        for i in y {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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

        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: RefCell::new(false),
                address: None,
                id: "y".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .unwrap();
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_while_loop() {
        let expr_loop = WhileLoop::<scope_impl::Scope>::parse(
            r##"
        while x > 10 {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_while() {
        let expr_loop = WhileLoop::<scope_impl::Scope>::parse(
            r##"
        while x {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_loop() {
        let expr_loop = Loop::<scope_impl::Scope>::parse(
            r##"
        loop {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
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
        let res = expr_loop.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }
}
