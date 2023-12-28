use std::rc::Rc;

use crate::ast::utils::strings::ID;

#[derive(Debug, Clone)]
pub enum SemanticError {}

pub trait Resolve<Scope: ScopeApi> {
    type Output;
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
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
        todo!()
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

// impl<User, Static> Resolve for EitherType<User, Static>
// where
//     User: CompatibleWith + TypeOf + Resolve,
//     Static: CompatibleWith + TypeOf + Resolve,
// {
//     type Output = ();
//     fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
//     where
//         Self: Sized,
//         Scope: ScopeApi,
//     {
//         match self {
//             EitherType::Static(static_type) => static_type.resolve(scope),
//             EitherType::User(user_type) => user_type.resolve(scope),
//         }
//     }
// }
pub trait ScopeApi
where
    Self: Sized,
{
    type UserType: CompatibleWith<Self> + TypeOf<Self> + Resolve<Self>;
    type StaticType: CompatibleWith<Self> + TypeOf<Self> + Resolve<Self>;
    type Fn: Resolve<Self>;
    type Var: Resolve<Self>;
    type Chan: Resolve<Self>;
    type Event: Resolve<Self>;

    fn register_type(&self, reg: Self::UserType) -> Result<(), SemanticError>;
    fn register_fn(&self, reg: Self::Fn) -> Result<(), SemanticError>;
    fn register_chan(&self, reg: Self::Chan) -> Result<(), SemanticError>;
    fn register_var(&self, reg: Self::Var) -> Result<(), SemanticError>;
    fn register_event(&self, reg: Self::Event) -> Result<(), SemanticError>;

    fn find_var(&self, id: &ID) -> Result<Self::Var, SemanticError>;
    fn find_fn(&self, id: &ID) -> Result<Self::Fn, SemanticError>;
    fn find_chan(&self) -> Result<Self::Chan, SemanticError>;
    fn find_type(id: &ID) -> Result<Self::UserType, SemanticError>;
    fn find_event(&self) -> Result<Self::Event, SemanticError>;
}
