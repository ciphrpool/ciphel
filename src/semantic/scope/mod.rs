use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::Rc,
};

use crate::ast::{statements::definition, types, utils::strings::ID};

use self::type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking};

use super::{CompatibleWith, EitherType, MergeType, SemanticError, TypeOf};
pub mod chan_impl;
pub mod event_impl;
pub mod scope_impl;
pub mod static_type_impl;
pub mod type_traits;
pub mod user_type_impl;
pub mod var_impl;

pub trait BuildStaticType<Scope: ScopeApi> {
    fn build_primitive(
        type_sig: &types::PrimitiveType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_slice(
        type_sig: &types::SliceType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_slice_from(
        size: &usize,
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_tuple(
        type_sig: &types::TupleType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_tuple_from(
        type_sig: &Vec<EitherType<Scope::UserType, Scope::StaticType>>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_vec(
        type_sig: &types::VecType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_vec_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_error() -> Scope::StaticType;

    fn build_fn(
        type_sig: &types::FnType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_fn_from(
        params: &Vec<EitherType<Scope::UserType, Scope::StaticType>>,
        ret: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_chan(
        type_sig: &types::ChanType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_chan_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_unit() -> Scope::StaticType;

    fn build_any() -> Scope::StaticType;

    fn build_addr(
        type_sig: &types::AddrType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_addr_from(
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;

    fn build_map(
        type_sig: &types::MapType,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
    fn build_map_from(
        key: &EitherType<Scope::UserType, Scope::StaticType>,
        value: &EitherType<Scope::UserType, Scope::StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<Scope::StaticType, SemanticError>;
}

pub trait BuildUserType<Scope: ScopeApi> {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<Scope::UserType, SemanticError>;
}

pub trait BuildVar<Scope: ScopeApi> {
    fn build_var(id: &ID, type_sig: &EitherType<Scope::UserType, Scope::StaticType>) -> Scope::Var;
}
pub trait BuildChan<Scope: ScopeApi> {
    fn build_chan(
        id: &ID,
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
    ) -> Scope::Chan;
}
pub trait BuildEvent<Scope: ScopeApi> {
    fn build_event(scope: &Ref<Scope>, event: &definition::EventDef<Scope>) -> Scope::Event;
}

pub trait ScopeApi
where
    Self: Sized + Clone + Debug,
{
    type UserType: Clone
        + Debug
        + CompatibleWith<Self>
        + TypeOf<Self>
        + BuildUserType<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + IsEnum
        + MergeType<Self>;
    type StaticType: Clone
        + Debug
        + CompatibleWith<Self>
        + TypeOf<Self>
        + BuildStaticType<Self>
        + GetSubTypes<Self>
        + TypeChecking<Self>
        + OperandMerging<Self>
        + MergeType<Self>;

    type Var: Clone + Debug + CompatibleWith<Self> + TypeOf<Self> + BuildVar<Self>;
    type Chan: CompatibleWith<Self> + TypeOf<Self> + BuildChan<Self>;
    type Event: BuildEvent<Self>;

    fn child_scope_with(
        parent: &Rc<RefCell<Self>>,
        vars: Vec<Self::Var>,
    ) -> Result<Rc<RefCell<Self>>, SemanticError>;

    fn register_type(&mut self, id: &ID, reg: Self::UserType) -> Result<(), SemanticError>;
    fn register_chan(&mut self, reg: &ID) -> Result<(), SemanticError>;
    fn register_var(&mut self, reg: Self::Var) -> Result<(), SemanticError>;
    fn register_event(&mut self, reg: Self::Event) -> Result<(), SemanticError>;

    fn find_var(&self, id: &ID) -> Result<Self::Var, SemanticError>;
    fn find_chan(&self) -> Result<&Self::Chan, SemanticError>;
    fn find_type(&self, id: &ID) -> Result<Self::UserType, SemanticError>;
    fn find_event(&self) -> Result<&Self::Event, SemanticError>;
}
