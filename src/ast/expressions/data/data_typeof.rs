use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::{type_traits::GetSubTypes, ScopeApi},
        EitherType, Resolve, SemanticError, TypeOf,
    },
};

use super::{
    Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, FieldAccess, KeyData,
    ListAccess, Map, MultiData, Primitive, PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable,
    Vector,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Data {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Data::Primitive(value) => value.type_of(scope),
            Data::Slice(value) => value.type_of(scope),
            Data::Vec(value) => value.type_of(scope),
            Data::Closure(value) => value.type_of(scope),
            Data::Chan(value) => value.type_of(scope),
            Data::Tuple(value) => value.type_of(scope),
            Data::Address(value) => value.type_of(scope),
            Data::PtrAccess(value) => value.type_of(scope),
            Data::Variable(value) => value.type_of(scope),
            Data::Unit => todo!(),
            Data::Map(value) => value.type_of(scope),
            Data::Struct(value) => value.type_of(scope),
            Data::Union(value) => value.type_of(scope),
            Data::Enum(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Variable {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Variable::Var(value) => value.type_of(scope),
            Variable::FieldAccess(value) => value.type_of(scope),
            Variable::ListAccess(value) => value.type_of(scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VarID {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var = scope.find_var(&self.0)?;
        var.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FieldAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.field.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ListAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(scope)?;
        Ok(<Option<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        > as GetSubTypes<Scope>>::get_item(&var_type))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for String {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Primitive {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Slice {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Vector {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Tuple {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MultiData {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Vec<(String, Expression)> {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for (&String, &Vec<Expression>) {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for (&String, &Vec<(String, Expression)>) {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Closure {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureScope {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureParam {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Address {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PtrAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Channel {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Struct {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Union {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Enum {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Map {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for KeyData {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
