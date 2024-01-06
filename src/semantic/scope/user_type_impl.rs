use std::{
    cell::Ref,
    collections::{HashMap, HashSet},
};

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
    pub id: ID,
    pub fields: HashMap<ID, EitherType<UserType, StaticType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub id: ID,
    pub values: HashSet<ID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub id: ID,
    pub variants: HashMap<ID, Struct>,
}

impl<Scope: ScopeApi<UserType = Self>> TypeOf<Scope> for UserType {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<Self, Scope::StaticType>, SemanticError>
    where
        Scope: super::ScopeApi,
        Self: Sized,
    {
        Ok(EitherType::User(self.clone()))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = Self>> CompatibleWith<Scope> for UserType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
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

impl<Scope: ScopeApi<StaticType = StaticType, UserType = Self>> BuildUserType<Scope> for UserType {
    fn build_usertype(
        type_sig: &crate::ast::statements::definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        match type_sig {
            definition::TypeDef::Struct(value) => {
                Ok(UserType::Struct(Struct::build(value, scope)?))
            }
            definition::TypeDef::Union(value) => Ok(UserType::Union(Union::build(value, scope)?)),
            definition::TypeDef::Enum(value) => Ok(UserType::Enum(Enum::build(value, scope)?)),
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = Self>> GetSubTypes<Scope> for UserType {
    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            UserType::Struct(value) => value.fields.get(field_id).map(|field| field.clone()),
            UserType::Enum(_) => None,
            UserType::Union(_) => None,
        }
    }
    fn get_variant(&self, variant: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            UserType::Struct(_) => None,
            UserType::Enum(value) => {
                if value.values.contains(variant) {
                    Some(EitherType::Static(StaticType::Unit))
                } else {
                    None
                }
            }
            UserType::Union(value) => value
                .variants
                .get(variant)
                .map(|field| EitherType::User(UserType::Struct(field.clone()))),
        }
    }
    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
        match self {
            UserType::Struct(value) => Some(
                value
                    .fields
                    .iter()
                    .map(|(key, val)| (Some(key.clone()), val.clone()))
                    .collect(),
            ),
            UserType::Enum(_) => None,
            UserType::Union(_) => None,
        }
    }
}

impl<Scope: ScopeApi<UserType = Self>> TypeChecking<Scope> for UserType {}

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
        scope: &Ref<Scope>,
    ) -> Result<EitherType<UserType, <Scope as ScopeApi>::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
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
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let EitherType::User(UserType::Struct(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.fields.len() != other_type.fields.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_key, self_field) in self.fields.iter() {
            if let Some(other_field) = other_type.fields.get(self_key) {
                let _ = self_field.compatible_with(other_field, scope)?;
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for Union
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let EitherType::User(UserType::Union(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.variants.len() != other_type.variants.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_variant_key, self_variant) in self.variants.iter() {
            if let Some(other_variant) = other_type.variants.get(self_variant_key) {
                if self_variant.fields.len() != other_variant.fields.len() {
                    return Err(SemanticError::IncompatibleTypes);
                }
                for (self_key, self_field) in self_variant.fields.iter() {
                    if let Some(other_field) = other_variant.fields.get(self_key) {
                        let _ = self_field.compatible_with(other_field, scope)?;
                    } else {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}
impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope> for Enum {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let EitherType::User(UserType::Enum(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.values.len() != other_type.values.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for id in &self.values {
            if !other_type.values.contains(id) {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl Struct {
    pub fn build<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>>(
        from: &definition::StructDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut fields = HashMap::new();
        for (id, field) in &from.fields {
            let field_type = field.type_of(&scope)?;
            fields.insert(id.clone(), field_type);
        }
        Ok(Self {
            id: from.id.clone(),
            fields,
        })
    }

    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}

impl Union {
    pub fn build<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>>(
        from: &definition::UnionDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut variants = HashMap::new();
        for (id, variant) in &from.variants {
            let mut fields = HashMap::new();
            for (id, field) in variant {
                let field_type = field.type_of(&scope)?;
                fields.insert(id.clone(), field_type);
            }
            variants.insert(
                id.clone(),
                Struct {
                    id: id.clone(),
                    fields,
                },
            );
        }
        Ok(Self {
            id: from.id.clone(),
            variants,
        })
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}

impl Enum {
    pub fn build<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>>(
        from: &definition::EnumDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut values = HashSet::new();
        for id in &from.values {
            values.insert(id.clone());
        }
        Ok(Self {
            id: from.id.clone(),
            values,
        })
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}
