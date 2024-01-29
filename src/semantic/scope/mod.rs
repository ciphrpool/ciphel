use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
};

use crate::ast::{statements::definition, types, utils::strings::ID};

use self::{
    chan_impl::Chan,
    event_impl::Event,
    static_types::StaticType,
    type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking},
    user_type_impl::UserType,
    var_impl::Var,
};

use super::{CompatibleWith, Either, MergeType, SemanticError, SizeOf, TypeOf};
pub mod chan_impl;
pub mod event_impl;
pub mod scope_impl;
pub mod static_types;
pub mod type_traits;
pub mod user_type_impl;
pub mod var_impl;

pub trait BuildStaticType<Scope: ScopeApi> {
    fn build_primitive(
        type_sig: &types::PrimitiveType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_slice(
        type_sig: &types::SliceType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_slice_from(
        size: &usize,
        type_sig: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_tuple(
        type_sig: &types::TupleType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_tuple_from(
        type_sig: &Vec<Either<UserType, StaticType>>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_vec(
        type_sig: &types::VecType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_vec_from(
        type_sig: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_error() -> StaticType;

    fn build_fn(type_sig: &types::FnType, scope: &Ref<Scope>) -> Result<StaticType, SemanticError>;
    fn build_fn_from(
        params: &Vec<Either<UserType, StaticType>>,
        ret: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_chan(
        type_sig: &types::ChanType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_chan_from(
        type_sig: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_unit() -> StaticType;

    fn build_any() -> StaticType;

    fn build_addr(
        type_sig: &types::AddrType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_addr_from(
        type_sig: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_map(
        type_sig: &types::MapType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_map_from(
        key: &Either<UserType, StaticType>,
        value: &Either<UserType, StaticType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
}

pub trait BuildUserType<Scope: ScopeApi> {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<UserType, SemanticError>;
}

pub trait BuildVar<Scope: ScopeApi> {
    fn build_var(id: &ID, type_sig: &Either<UserType, StaticType>) -> Var;
}
pub trait BuildChan<Scope: ScopeApi> {
    fn build_chan(id: &ID, type_sig: &Either<UserType, StaticType>) -> Chan;
}
pub trait BuildEvent<Scope: ScopeApi> {
    fn build_event(scope: &Ref<Scope>, event: &definition::EventDef<Scope>) -> Event;
}

pub trait ScopeApi
where
    Self: Sized + Clone + Debug,
{
    // type UserType: Clone
    //     + Debug
    //     + CompatibleWith<Self>
    //     + TypeOf<Self>
    //     + BuildUserType<Self>
    //     + GetSubTypes<Self>
    //     + TypeChecking<Self>
    //     + OperandMerging<Self>
    //     + IsEnum
    //     + MergeType<Self>
    //     + PartialEq
    //     + SizeOf;

    // type StaticType: Clone
    //     + Debug
    //     + CompatibleWith<Self>
    //     + TypeOf<Self>
    //     + BuildStaticType<Self>
    //     + GetSubTypes<Self>
    //     + TypeChecking<Self>
    //     + OperandMerging<Self>
    //     + MergeType<Self>
    //     + PartialEq
    //     + SizeOf;

    // type Var: Clone + Debug + CompatibleWith<Self> + TypeOf<Self> + BuildVar<Self> + PartialEq;
    // type Chan: CompatibleWith<Self> + TypeOf<Self> + BuildChan<Self>;
    // type Event: BuildEvent<Self>;

    fn child_scope_with(
        parent: &Rc<RefCell<Self>>,
        vars: Vec<Var>,
    ) -> Result<Rc<RefCell<Self>>, SemanticError>;

    fn register_type(&mut self, id: &ID, reg: UserType) -> Result<(), SemanticError>;
    fn register_chan(&mut self, reg: &ID) -> Result<(), SemanticError>;
    fn register_var(&mut self, reg: Var) -> Result<(), SemanticError>;
    fn register_event(&mut self, reg: Event) -> Result<(), SemanticError>;

    fn find_var(&self, id: &ID) -> Result<Rc<Var>, SemanticError>;
    fn find_outer_vars(&self) -> HashMap<ID, Rc<Var>>;
    fn find_chan(&self) -> Result<&Chan, SemanticError>;
    fn find_type(&self, id: &ID) -> Result<Rc<UserType>, SemanticError>;
    fn find_event(&self) -> Result<&Event, SemanticError>;
}
