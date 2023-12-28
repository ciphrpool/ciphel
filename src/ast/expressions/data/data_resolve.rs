use crate::semantic::{Resolve, ScopeApi, SemanticError};

use super::{
    Access, Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, KeyData, Map,
    MultiData, Primitive, Slice, Struct, Tuple, Union, Variable, Vector,
};

impl<Scope: ScopeApi> Resolve<Scope> for Data {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope),
            Data::Slice(value) => value.resolve(scope),
            Data::Vec(value) => value.resolve(scope),
            Data::Closure(value) => value.resolve(scope),
            Data::Chan(value) => value.resolve(scope),
            Data::Tuple(value) => value.resolve(scope),
            Data::Address(value) => value.resolve(scope),
            Data::Access(value) => value.resolve(scope),
            Data::Variable(value) => value.resolve(scope),
            Data::Unit => todo!(),
            Data::Map(value) => value.resolve(scope),
            Data::Struct(value) => value.resolve(scope),
            Data::Union(value) => value.resolve(scope),
            Data::Enum(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Variable {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(value) => {
                let _ = scope.find_var(value)?;

                Ok(())
            }
            Variable::FieldAccess(_) => todo!(),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Primitive {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Slice {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Vector {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Tuple {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MultiData {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureScope {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureParam {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Address {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Access {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Channel {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Struct {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Union {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Enum {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Map {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for KeyData {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
