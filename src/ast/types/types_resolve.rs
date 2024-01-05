use std::{cell::RefCell, rc::Rc};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};
use crate::semantic::{
    scope::{type_traits::IsEnum, ScopeApi},
    Resolve, SemanticError,
};

impl<Scope: ScopeApi> Resolve<Scope> for Type {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Type::Primitive(value) => value.resolve(scope, context),
            Type::Slice(value) => value.resolve(scope, context),
            Type::UserType(value) => {
                let _ = scope.borrow().find_type(value)?;
                Ok(())
            }
            Type::Vec(value) => value.resolve(scope, context),
            Type::Fn(value) => value.resolve(scope, context),
            Type::Chan(value) => value.resolve(scope, context),
            Type::Tuple(value) => value.resolve(scope, context),
            Type::Unit => Ok(()),
            Type::Address(value) => value.resolve(scope, context),
            Type::Map(value) => value.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PrimitiveType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
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

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            SliceType::String => Ok(()),
            SliceType::List(_, subtype) => subtype.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for VecType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for param in &self.params {
            let _ = param.resolve(scope, context)?;
        }
        self.ret.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Types {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for item in self {
            let _ = item.resolve(scope, context)?;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ChanType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TupleType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AddrType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for MapType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.keys_type.resolve(scope, context)?;
        self.values_type.resolve(scope, context)
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for KeyType {
    type Output = ();
    type Context = ();

    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            KeyType::Primitive(value) => value.resolve(scope, context),
            KeyType::Address(value) => value.resolve(scope, context),
            KeyType::Slice(value) => value.resolve(scope, context),
            KeyType::EnumID(value) => {
                let borrowed_scope = scope.borrow();
                let enum_type: <Scope as ScopeApi>::UserType = borrowed_scope.find_type(value)?;
                if enum_type.is_enum() {
                    return Err(SemanticError::ExpectedEnum);
                }
                Ok(())
            }
        }
    }
}
