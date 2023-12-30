use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};
use crate::semantic::scope::BuildStaticType;
use crate::{
    ast::statements::scope::Scope,
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf},
};

impl<Scope: ScopeApi> TypeOf<Scope> for Type {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Type::Primitive(value) => value.type_of(scope),
            Type::Slice(value) => value.type_of(scope),
            Type::UserType(value) => {
                let user_type = scope.find_type(value)?;
                let user_type = user_type.type_of(scope)?;
                Ok(user_type)
            }
            Type::Vec(value) => value.type_of(scope),
            Type::Fn(value) => value.type_of(scope),
            Type::Chan(value) => value.type_of(scope),
            Type::Tuple(value) => value.type_of(scope),
            Type::Unit => {
                let static_type: Scope::StaticType = Scope::StaticType::build_unit();
                let static_type = static_type.type_of(scope)?;
                Ok(static_type)
            }
            Type::Address(value) => value.type_of(scope),
            Type::Map(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PrimitiveType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_primitive(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for SliceType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_slice(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VecType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_vec(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FnType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_fn(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ChanType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_chan(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for TupleType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_tuple(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for AddrType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_addr(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for MapType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_map(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}
