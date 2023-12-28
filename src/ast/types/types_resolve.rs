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

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PrimitiveType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for SliceType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for VecType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Types {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for ChanType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TupleType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AddrType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for MapType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for KeyType {
    type Output = ();

    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}
