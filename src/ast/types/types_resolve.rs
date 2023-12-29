use crate::{
    ast::statements::scope::Scope,
    semantic::{Resolve, ScopeApi, SemanticError},
};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};

impl<Scope: ScopeApi> Resolve<Scope> for Type {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PrimitiveType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for SliceType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for VecType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Types {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ChanType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TupleType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AddrType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for MapType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for KeyType {
    type Output = ();
    type Context = ();

    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
