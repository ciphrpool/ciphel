use std::collections::{HashMap, HashSet};

use crate::{
    ast::{
        statements::definition::{self, StructDef},
        utils::strings::ID,
    },
    semantic::{CompatibleWith, EitherType, MergeType, SemanticError, TypeOf},
};

use super::{
    static_type_impl::StaticType,
    type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking},
    BuildUserType, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UserType {
    Struct(Struct),
    Enum(Enum),
    Union(Union),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    id: ID,
    fields: HashMap<ID, EitherType<UserType, StaticType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    id: ID,
    values: HashSet<ID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    id: ID,
    variants: HashMap<ID, HashMap<ID, EitherType<UserType, StaticType>>>,
}

impl<Scope: ScopeApi<UserType = Self>> TypeOf<Scope> for UserType {
    fn type_of(&self, _scope: &Scope) -> Result<EitherType<Self, Scope::StaticType>, SemanticError>
    where
        Scope: super::ScopeApi,
        Self: Sized,
    {
        Ok(EitherType::User(self.clone()))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = Self>> CompatibleWith<Scope> for UserType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            UserType::Struct(value) => value.compatible_with(other, scope),
            UserType::Enum(value) => value.compatible_with(other, scope),
            UserType::Union(value) => value.compatible_with(other, scope),
        }
    }
}

impl<Scope: ScopeApi<UserType = Self>> BuildUserType<Scope> for UserType {
    fn build_usertype(type_sig: &crate::ast::statements::definition::TypeDef) -> Self {
        match type_sig {
            definition::TypeDef::Struct(value) => UserType::Struct(Struct::build(value)),
            definition::TypeDef::Union(value) => UserType::Union(Union::build(value)),
            definition::TypeDef::Enum(value) => UserType::Enum(Enum::build(value)),
        }
    }
}

impl<Scope: ScopeApi<UserType = Self>> GetSubTypes<Scope> for UserType {
    fn get_nth(&self, _n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        todo!()
    }
    fn get_field(&self, _field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        todo!()
    }
    fn get_variant(&self, _variant: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        todo!()
    }
    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        todo!()
    }
    fn get_return(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        todo!()
    }
    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = Self>> TypeChecking<Scope> for UserType {
    fn is_enum_variant(&self) -> bool {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = Self>> OperandMerging<Scope> for UserType {}

impl IsEnum for UserType {
    fn is_enum(&self) -> bool {
        match self {
            UserType::Struct(_) => false,
            UserType::Enum(_) => true,
            UserType::Union(_) => false,
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for UserType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<UserType, <Scope as ScopeApi>::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::User(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            UserType::Struct(value) => match other_type {
                UserType::Struct(other_type) => value
                    .merge(&other_type)
                    .map(|v| EitherType::User(UserType::Struct(v))),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            UserType::Enum(value) => match other_type {
                UserType::Enum(other_type) => value
                    .merge(&other_type)
                    .map(|v| EitherType::User(UserType::Enum(v))),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            UserType::Union(value) => match other_type {
                UserType::Union(other_type) => value
                    .merge(&other_type)
                    .map(|v| EitherType::User(UserType::Union(v))),
                _ => Err(SemanticError::IncompatibleTypes),
            },
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for Struct
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for Union
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}
impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope> for Enum {
    fn compatible_with<Other>(&self, _other: &Other, _scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl Struct {
    pub fn build(from: &definition::StructDef) -> Self {
        todo!()
    }

    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        todo!()
    }
}

impl Union {
    pub fn build(from: &definition::UnionDef) -> Self {
        todo!()
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        todo!()
    }
}

impl Enum {
    pub fn build(from: &definition::EnumDef) -> Self {
        todo!()
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        todo!()
    }
}
