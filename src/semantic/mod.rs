use std::rc::Rc;

use crate::ast::utils::strings::ID;

#[derive(Debug, Clone)]
pub enum SemanticError {}

pub trait Resolve {
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi;
}

pub trait CompatibleWith {
    fn compatible_with<Other, Scope>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
        Scope: ScopeApi;
}

pub trait TypeOf {
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve;
}

impl<User, Static> CompatibleWith for Option<EitherType<User, Static>>
where
    User: CompatibleWith + TypeOf + Resolve,
    Static: CompatibleWith + TypeOf + Resolve,
{
    fn compatible_with<Other, Scope>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
        Scope: ScopeApi,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EitherType<User, Static> {
    Static(Static),
    User(User),
}

impl<User, Static> CompatibleWith for EitherType<User, Static>
where
    User: CompatibleWith + TypeOf + Resolve,
    Static: CompatibleWith + TypeOf + Resolve,
{
    fn compatible_with<Other, Scope>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<(), SemanticError>
    where
        Other: TypeOf,
        Scope: ScopeApi,
        Self: TypeOf,
    {
        match self {
            EitherType::Static(static_type) => static_type.compatible_with(other, scope),
            EitherType::User(user_type) => user_type.compatible_with(other, scope),
        }
    }
}

impl<User, Static> TypeOf for EitherType<User, Static>
where
    User: CompatibleWith + TypeOf + Resolve,
    Static: CompatibleWith + TypeOf + Resolve,
{
    fn type_of<Scope>(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve,
    {
        match self {
            EitherType::Static(static_type) => static_type.type_of(scope),
            EitherType::User(user_type) => user_type.type_of(scope),
        }
    }
}

impl<User, Static> Resolve for EitherType<User, Static>
where
    User: CompatibleWith + TypeOf + Resolve,
    Static: CompatibleWith + TypeOf + Resolve,
{
    fn resolve<Scope>(&self, scope: &Scope) -> Result<(), SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            EitherType::Static(static_type) => static_type.resolve(scope),
            EitherType::User(user_type) => user_type.resolve(scope),
        }
    }
}
pub trait ScopeApi {
    type UserType: CompatibleWith + TypeOf + Resolve;
    type StaticType: CompatibleWith + TypeOf + Resolve;
    type Fn: Resolve;
    type Var: Resolve;
    type Chan: Resolve;
    type Event: Resolve;

    fn register_type(reg: Self::UserType) -> Result<(), SemanticError>;
    fn register_fn(reg: Self::Fn) -> Result<(), SemanticError>;
    fn register_chan(reg: Self::Chan) -> Result<(), SemanticError>;
    fn register_var(reg: Self::Var) -> Result<(), SemanticError>;
    fn register_event(reg: Self::Event) -> Result<(), SemanticError>;

    fn find_var(id: ID) -> Result<Self::Var, SemanticError>;
    fn find_fn(id: ID) -> Result<Self::Fn, SemanticError>;
    fn find_chan() -> Result<Self::Chan, SemanticError>;
    fn find_type(id: ID) -> Result<Self::UserType, SemanticError>;
    fn find_event() -> Result<Self::Event, SemanticError>;
}
