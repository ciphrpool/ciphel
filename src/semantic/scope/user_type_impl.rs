use crate::semantic::{CompatibleWith, EitherType, MergeType, Resolve, SemanticError, TypeOf};

use super::{
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
pub struct Struct();

#[derive(Debug, Clone, PartialEq)]
pub struct Enum();

#[derive(Debug, Clone, PartialEq)]
pub struct Union();

impl<Scope: ScopeApi<UserType = Self>> TypeOf<Scope> for UserType {
    fn type_of(&self, scope: &Scope) -> Result<EitherType<Self, Scope::StaticType>, SemanticError>
    where
        Scope: super::ScopeApi,
        Self: Sized,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = Self>> CompatibleWith<Scope> for UserType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = Self>> BuildUserType<Scope> for UserType {
    fn build_usertype(type_sig: &crate::ast::statements::definition::TypeDef) -> Self {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = Self>> GetSubTypes<Scope> for UserType {}

impl<Scope: ScopeApi<UserType = Self>> TypeChecking<Scope> for UserType {}

impl<Scope: ScopeApi<UserType = Self>> OperandMerging<Scope> for UserType {}

impl IsEnum for UserType {}

impl<Scope: ScopeApi<UserType = UserType>> MergeType<Scope> for UserType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<UserType, <Scope as ScopeApi>::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl<Scope: ScopeApi<UserType = UserType>> CompatibleWith<Scope> for Enum {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}

impl Enum {
    pub fn merge(&self, other: &Enum) -> Result<Enum, SemanticError> {
        todo!()
    }
}
