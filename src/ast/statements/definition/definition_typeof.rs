use super::{
    Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, StructVariant, TypeDef,
    UnionDef, UnionVariant,
};
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

// impl<Scope: ScopeApi> TypeOf<Scope> for Definition {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         match self {
//             Definition::Type(value) => value.type_of(scope),
//             Definition::Fn(value) => value.type_of(scope),
//             Definition::Event(value) => value.type_of(scope),
//         }
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for TypeDef {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         Ok(Some(EitherType::User(Scope::UserType::build_user_type(
//             self,
//         ))))
//         // match self {
//         //     TypeDef::Struct(value) => value.type_of(scope),
//         //     TypeDef::Union(value) => value.type_of(scope),
//         //     TypeDef::Enum(value) => value.type_of(scope),
//         // }
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for StructVariant {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         todo!()
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for StructDef {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         todo!()
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for UnionVariant {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         todo!()
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for UnionDef {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         todo!()
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for EnumDef {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         todo!()
//     }
// }

// impl<Scope: ScopeApi> TypeOf<Scope> for FnDef {
//     fn type_of(
//         &self,
//         scope: &Scope,
//     ) -> Result<
//         Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
//         SemanticError,
//     >
//     where
//         Scope: ScopeApi,
//         Self: Sized + Resolve<Scope>,
//     {
//         self.ret.type_of(scope)
//     }
// }

impl<Scope: ScopeApi> TypeOf<Scope> for EventDef {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for EventCondition {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<
        Option<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>>,
        SemanticError,
    >
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        todo!()
    }
}
