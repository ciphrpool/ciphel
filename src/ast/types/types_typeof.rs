use crate::{
    ast::statements::scope::Scope,
    semantic::{scope::ScopeApi, SemanticError, TypeOf},
};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, PrimitiveType, SliceType, TupleType, Type, Types,
    VecType,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Type {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PrimitiveType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for SliceType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VecType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FnType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Types {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ChanType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for TupleType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for AddrType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for MapType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for KeyType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<
            crate::semantic::EitherType<
                <Scope as ScopeApi>::UserType,
                <Scope as ScopeApi>::StaticType,
            >,
        >,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + crate::semantic::Resolve<Scope>,
    {
        todo!()
    }
}
