use std::rc::Rc;

use crate::ast::{statements::definition, utils::strings::ID};

#[derive(Debug, Clone)]
pub enum SemanticError {
    CantInferType,
    ExpectBoolean,
    ExpectIterable,
    UnknownField,
    IncorrectVariant,
    InvalidPattern,
}

pub trait Resolve<Scope: ScopeApi> {
    type Output;
    type Context;
    fn resolve(
        &self,
        scope: &Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized;
}

pub trait CompatibleWith<Scope: ScopeApi> {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait TypeOf<Scope: ScopeApi> {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>;
}

impl<User, Static, Scope: ScopeApi> CompatibleWith<Scope> for Option<EitherType<User, Static>>
where
    User: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
    Static: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
        Scope: ScopeApi,
    {
        match self {
            Some(inner) => inner.compatible_with(other, scope),
            None => Ok(()), // TODO : Verify because maybe Err(SemanticError::CantInferType),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EitherType<User, Static> {
    Static(Static),
    User(User),
}

impl<User, Static, Scope: ScopeApi> CompatibleWith<Scope> for EitherType<User, Static>
where
    User: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
    Static: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
        Scope: ScopeApi,
        Self: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(static_type) => static_type.compatible_with(other, scope),
            EitherType::User(user_type) => user_type.compatible_with(other, scope),
        }
    }
}

impl<User, Static, Scope: ScopeApi> TypeOf<Scope> for EitherType<User, Static>
where
    User: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
    Static: CompatibleWith<Scope> + TypeOf<Scope> + Resolve<Scope>,
{
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            EitherType::Static(static_type) => static_type.type_of(scope),
            EitherType::User(user_type) => user_type.type_of(scope),
        }
    }
}

pub trait MergeType<Scope: ScopeApi> {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>;
}

pub trait BuildFn<Scope: ScopeApi> {
    fn build_fn(func: &definition::FnDef) -> Scope::Fn;
}

pub trait BuildType<Scope: ScopeApi> {
    fn build_type(type_sig: &definition::TypeDef) -> Scope::UserType;
}

pub trait BuildVar<Scope: ScopeApi> {
    fn build_var(id: &ID, type_sig: &EitherType<Scope::UserType, Scope::StaticType>) -> Scope::Var;
}
pub trait BuildChan<Scope: ScopeApi> {
    fn build_chan(id: &ID, type_sig: &EitherType<Scope::UserType, Scope::StaticType>)
        -> Scope::Var;
}

pub trait BuildEvent<Scope: ScopeApi> {
    fn build_event(scope: &Scope, event: &definition::FnDef) -> Scope::Var;
}

pub trait RetrieveTypeInfo<Scope: ScopeApi> {
    fn is_iterable(&self) -> bool {
        false
    }
    fn is_boolean(&self) -> bool {
        false
    }
    fn is_enum_variant(&self) -> bool {
        false
    }
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_variant(&self, variant: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn iter_on_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
        None
    }
}
impl<Scope: ScopeApi, T: RetrieveTypeInfo<Scope>> RetrieveTypeInfo<Scope> for Option<T> {
    fn get_variant(
        &self,
        variant: &ID,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            Some(value) => value.get_variant(variant),
            None => None,
        }
    }
    fn get_field(
        &self,
        field_id: &ID,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            Some(value) => value.get_field(field_id),
            None => None,
        }
    }
    fn get_nth(
        &self,
        n: &usize,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            Some(value) => value.get_nth(n),
            None => None,
        }
    }

    fn iter_on_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        )>,
    > {
        match self {
            Some(value) => value.iter_on_fields(),
            None => None,
        }
    }

    fn get_item(
        &self,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            Some(value) => value.get_item(),
            None => None,
        }
    }
    fn is_boolean(&self) -> bool {
        match self {
            Some(value) => value.is_boolean(),
            None => false,
        }
    }
    fn is_iterable(&self) -> bool {
        match self {
            Some(value) => value.is_iterable(),
            None => false,
        }
    }
    fn is_enum_variant(&self) -> bool {
        match self {
            Some(value) => value.is_enum_variant(),
            None => false,
        }
    }
}
impl<Scope: ScopeApi> RetrieveTypeInfo<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn is_iterable(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_iterable(),
            EitherType::User(_) => false,
        }
    }

    fn is_boolean(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_boolean(),
            EitherType::User(_) => false,
        }
    }
    fn is_enum_variant(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_enum_variant(),
            EitherType::User(user_type) => user_type.is_enum_variant(),
        }
    }

    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(static_type) => static_type.get_nth(n),
            EitherType::User(user_type) => user_type.get_nth(n),
        }
    }

    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(_) => None,
            EitherType::User(user_type) => user_type.get_field(field_id),
        }
    }
    fn get_variant(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(_) => None,
            EitherType::User(user_type) => user_type.get_field(field_id),
        }
    }
    fn get_item(
        &self,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            EitherType::Static(static_type) => static_type.get_item(),
            EitherType::User(_) => None,
        }
    }
    fn iter_on_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        )>,
    > {
        match self {
            EitherType::Static(static_type) => static_type.iter_on_fields(),
            EitherType::User(user_type) => user_type.iter_on_fields(),
        }
    }
}

pub trait ScopeApi
where
    Self: Sized,
{
    type UserType: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildType<Self>
        + RetrieveTypeInfo<Self>;
    type StaticType: CompatibleWith<Self> + TypeOf<Self> + Resolve<Self> + RetrieveTypeInfo<Self>;
    type Fn: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildFn<Self>
        + RetrieveTypeInfo<Self>;
    type Var: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildVar<Self>
        + RetrieveTypeInfo<Self>;
    type Chan: CompatibleWith<Self> + TypeOf<Self> + Resolve<Self> + BuildChan<Self>;
    type Event: Resolve<Self> + BuildEvent<Self>;

    fn child_scope(&self) -> Result<Self, SemanticError>;
    fn attach(&mut self, vars: impl Iterator<Item = Self::Var>);

    fn register_type(&self, reg: Self::UserType) -> Result<(), SemanticError>;
    fn register_fn(&self, reg: Self::Fn) -> Result<(), SemanticError>;
    fn register_chan(&self, reg: &ID) -> Result<(), SemanticError>;
    fn register_var(&self, reg: Self::Var) -> Result<(), SemanticError>;
    fn register_event(&self, reg: Self::Event) -> Result<(), SemanticError>;

    fn find_var(&self, id: &ID) -> Result<&Self::Var, SemanticError>;
    fn find_fn(&self, id: &ID) -> Result<&Self::Fn, SemanticError>;
    fn find_chan(&self) -> Result<&Self::Chan, SemanticError>;
    fn find_type(&self, id: &ID) -> Result<&Self::UserType, SemanticError>;
    fn find_event(&self) -> Result<&Self::Event, SemanticError>;
}
