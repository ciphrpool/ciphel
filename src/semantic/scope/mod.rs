use std::{
    cell::{Ref},
    fmt::Debug,
};

use crate::{
    ast::{statements::definition, types, utils::strings::ID},
};

use self::{
    chan_impl::Chan,
    event_impl::Event,
    static_types::{NumberType, StaticType},
    user_type_impl::UserType,
    var_impl::Var,
};
use crate::semantic::scope::scope_impl::Scope;

use super::{EType, SemanticError};
pub mod chan_impl;
pub mod event_impl;
pub mod scope_impl;
pub mod static_types;
pub mod type_traits;
pub mod user_type_impl;
pub mod var_impl;

pub trait BuildStaticType {
    fn build_primitive(
        type_sig: &types::PrimitiveType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_str_slice(
        type_sig: &types::StrSliceType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_string(
        type_sig: &types::StringType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_slice(
        type_sig: &types::SliceType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_slice_from(
        size: &usize,
        type_sig: &EType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_tuple(
        type_sig: &types::TupleType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_tuple_from(
        type_sig: &Vec<EType>,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_vec(
        type_sig: &types::VecType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_vec_from(type_sig: &EType, scope: &Ref<Scope>) -> Result<StaticType, SemanticError>;

    fn build_error() -> StaticType;

    fn build_range(
        type_sig: &types::RangeType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_range_from(
        type_sig: NumberType,
        inclusiv: bool,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_fn(
        params: &Vec<EType>,
        ret: &EType,
        scope_params_size: usize,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_closure(
        params: &Vec<EType>,
        ret: &EType,
        closed: bool,
        scope_params_size: usize,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_chan(
        type_sig: &types::ChanType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_chan_from(type_sig: &EType, scope: &Ref<Scope>) -> Result<StaticType, SemanticError>;

    fn build_unit() -> StaticType;

    fn build_any() -> StaticType;

    fn build_addr(
        type_sig: &types::AddrType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_addr_from(type_sig: &EType, scope: &Ref<Scope>) -> Result<StaticType, SemanticError>;

    fn build_map(
        type_sig: &types::MapType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_map_from(
        key: &EType,
        value: &EType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
}

pub trait BuildUserType {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<UserType, SemanticError>;
}

pub trait BuildVar {
    fn build_var(id: &ID, type_sig: &EType) -> Var;
}
pub trait BuildChan {
    fn build_chan(id: &ID, type_sig: &EType) -> Chan;
}
pub trait BuildEvent {
    fn build_event(scope: &Ref<Scope>, event: &definition::EventDef) -> Event;
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ClosureState {
    CAPTURING,
    NOT_CAPTURING,
    DEFAULT,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ScopeState {
    pub is_closure: ClosureState,
    pub is_generator: bool,
    pub is_loop: bool,
}

impl Default for ScopeState {
    fn default() -> Self {
        Self {
            is_closure: ClosureState::DEFAULT,
            is_generator: false,
            is_loop: false,
        }
    }
}
