use std::{cell::RefCell, rc::Rc};

use super::{
    AddrType, ChanType, ClosureType, KeyType, MapType, PrimitiveType, RangeType, SliceType,
    StrSliceType, StringType, TupleType, Type, Types, VecType,
};
use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::{scope::type_traits::IsEnum, MutRc, Resolve, SemanticError};

impl Resolve for Type {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Type::Primitive(value) => value.resolve(scope, context, extra),
            Type::Slice(value) => value.resolve(scope, context, extra),
            Type::StrSlice(value) => value.resolve(scope, context, extra),
            Type::UserType(value) => {
                let _ = scope.borrow().find_type(value)?;
                Ok(())
            }
            Type::Vec(value) => value.resolve(scope, context, extra),
            Type::Closure(value) => value.resolve(scope, context, extra),
            Type::Chan(value) => value.resolve(scope, context, extra),
            Type::Tuple(value) => value.resolve(scope, context, extra),
            Type::Unit => Ok(()),
            Type::Any => Ok(()),
            Type::Address(value) => value.resolve(scope, context, extra),
            Type::Map(value) => value.resolve(scope, context, extra),
            Type::String(value) => value.resolve(scope, context, extra),
            Type::Range(value) => value.resolve(scope, context, extra),
        }
    }
}
impl Resolve for PrimitiveType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
impl Resolve for SliceType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.item_type.resolve(scope, context, extra)
    }
}

impl Resolve for StrSliceType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for StringType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for VecType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context, extra)
    }
}

impl Resolve for ClosureType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
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

impl Resolve for Types {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
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

impl Resolve for ChanType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context, extra)
    }
}

impl Resolve for TupleType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context, extra)
    }
}

impl Resolve for AddrType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.0.resolve(scope, context, extra)
    }
}
impl Resolve for RangeType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for MapType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
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

impl Resolve for KeyType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            KeyType::Primitive(value) => value.resolve(scope, context, extra),
            KeyType::Address(value) => value.resolve(scope, context, extra),
            KeyType::String(value) => Ok(()),
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
