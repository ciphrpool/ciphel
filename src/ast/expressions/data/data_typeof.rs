use crate::semantic::{EitherType, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{
    Access, Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, KeyData, Map,
    MultiData, Primitive, Slice, Struct, Tuple, Union, Variable, Vector,
};

impl TypeOf for Data {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Variable {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Primitive {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Slice {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Vector {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Tuple {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for MultiData {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Closure {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for ClosureScope {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for ClosureParam {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Access {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Channel {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Struct {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Enum {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for Map {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
impl TypeOf for KeyData {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        todo!()
    }
}
