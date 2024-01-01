use crate::ast::{
    expressions::{error, Expression},
    statements::definition,
    types,
    utils::strings::ID,
};

use self::type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking};

use super::{CompatibleWith, EitherType, MergeType, Resolve, SemanticError, TypeOf};

pub mod type_traits;

pub trait BuildStaticType<Scope: ScopeApi> {
    fn build_primitive(type_sig: &types::PrimitiveType) -> Scope::StaticType;

    fn build_slice(type_sig: &types::SliceType) -> Scope::StaticType;
    fn build_slice_from(
        type_sig: &Vec<EitherType<Scope::UserType, Scope::StaticType>>,
    ) -> Scope::StaticType;

    fn build_tuple(type_sig: &types::TupleType) -> Scope::StaticType;
    fn build_tuple_from(
        type_sig: &Vec<EitherType<Scope::UserType, Scope::StaticType>>,
    ) -> Scope::StaticType;

    fn build_vec(type_sig: &types::VecType) -> Scope::StaticType;
    fn build_vec_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;

    fn build_error(type_sig: &error::Error) -> Scope::StaticType;

    fn build_fn(type_sig: &types::FnType) -> Scope::StaticType;
    fn build_fn_from(
        params: &Vec<EitherType<Scope::UserType, Scope::StaticType>>,
        ret: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;

    fn build_chan(type_sig: &types::ChanType) -> Scope::StaticType;
    fn build_chan_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;

    fn build_unit() -> Scope::StaticType;

    fn build_any() -> Scope::StaticType;

    fn build_addr(type_sig: &types::AddrType) -> Scope::StaticType;
    fn build_addr_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;

    fn build_ptr_access_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;

    fn build_map(type_sig: &types::MapType) -> Scope::StaticType;
    fn build_map_from(
        key: &EitherType<Scope::UserType, Scope::StaticType>,
        value: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::StaticType;
}

pub trait BuildUserType<Scope: ScopeApi> {
    fn build_usertype(type_sig: &definition::TypeDef) -> Scope::UserType;
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

pub trait ScopeApi
where
    Self: Sized,
{
    type UserType: CompatibleWith<Self>
        + TypeOf<Self>
        + Resolve<Self>
        + BuildUserType<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + IsEnum
        + MergeType<Self>;
    type StaticType: CompatibleWith<Self>
        + TypeOf<Self>
        + BuildStaticType<Self>
        + Resolve<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + MergeType<Self>;

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
    fn register_chan(&self, reg: &ID) -> Result<(), SemanticError>;
    fn register_var(&self, reg: Self::Var) -> Result<(), SemanticError>;
    fn register_event(&self, reg: Self::Event) -> Result<(), SemanticError>;

    fn find_var(&self, id: &ID) -> Result<&Self::Var, SemanticError>;
    fn find_chan(&self) -> Result<&Self::Chan, SemanticError>;
    fn find_type(&self, id: &ID) -> Result<&Self::UserType, SemanticError>;
    fn find_event(&self) -> Result<&Self::Event, SemanticError>;
}
