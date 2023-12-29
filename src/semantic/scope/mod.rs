use crate::ast::utils::strings::ID;

use self::type_traits::{GetSubTypes, OperandMerging, TypeChecking};

use super::{
    BuildChan, BuildEvent, BuildFn, BuildType, BuildVar, CompatibleWith, MergeType, Resolve,
    SemanticError, TypeOf,
};

pub mod type_traits;

pub trait ScopeApi
where
    Self: Sized,
{
    type UserType: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildType<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + MergeType<Self>;
    type StaticType: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + MergeType<Self>;
    type Fn: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildFn<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>;
    type Var: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildVar<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>;
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
