use std::{cell::RefCell, rc::Rc};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};
use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, static_types::StaticType, type_traits::IsEnum,
        user_type_impl::UserType, var_impl::Var, ScopeApi,
    },
    Resolve, SemanticError,
};

impl<Scope: ScopeApi> Resolve<Scope> for Type {
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
    {
        match self {
            Type::Primitive(value) => value.resolve(scope, context, extra),
            Type::Slice(value) => value.resolve(scope, context, extra),
            Type::UserType(value) => {
                let _ = scope.borrow().find_type(value)?;
                Ok(())
            }
            Type::Vec(value) => value.resolve(scope, context, extra),
            Type::Fn(value) => value.resolve(scope, context, extra),
            Type::Chan(value) => value.resolve(scope, context, extra),
            Type::Tuple(value) => value.resolve(scope, context, extra),
            Type::Unit => Ok(()),
            Type::Address(value) => value.resolve(scope, context, extra),
            Type::Map(value) => value.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PrimitiveType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for SliceType {
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
    {
        match self {
            SliceType::String => Ok(()),
            SliceType::List(_, subtype) => subtype.resolve(scope, context, extra),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for VecType {
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
    {
        self.0.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnType {
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
    {
        for param in &self.params {
            let _ = param.resolve(scope, context, extra)?;
        }
        self.ret.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Types {
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
    {
        for item in self {
            let _ = item.resolve(scope, context, extra)?;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ChanType {
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
    {
        self.0.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TupleType {
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
    {
        self.0.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AddrType {
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
    {
        self.0.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for MapType {
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
    {
        let _ = self.keys_type.resolve(scope, context, extra)?;
        self.values_type.resolve(scope, context, extra)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for KeyType {
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
    {
        match self {
            KeyType::Primitive(value) => value.resolve(scope, context, extra),
            KeyType::Address(value) => value.resolve(scope, context, extra),
            KeyType::Slice(value) => value.resolve(scope, context, extra),
            KeyType::EnumID(value) => {
                let borrowed_scope = scope.borrow();
                let enum_type = borrowed_scope.find_type(value)?;
                if enum_type.is_enum() {
                    return Err(SemanticError::ExpectedEnum);
                }
                Ok(())
            }
        }
    }
}
