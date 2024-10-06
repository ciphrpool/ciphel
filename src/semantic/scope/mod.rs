use std::fmt::Debug;

use crate::ast::{statements::definition, types, utils::strings::ID};

use self::{
    static_types::{NumberType, StaticType},
    user_type_impl::UserType,
    var_impl::Var,
};
use crate::semantic::scope::scope::Scope;

use super::{EType, SemanticError};
pub mod scope;
pub mod static_types;
pub mod type_traits;
pub mod user_type_impl;
pub mod var_impl;

pub trait BuildStaticType {
    fn build_primitive(
        type_sig: &types::PrimitiveType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_str_slice(
        type_sig: &types::StrSliceType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_string(
        type_sig: &types::StringType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_slice(
        type_sig: &types::SliceType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_slice_from(
        size: &usize,
        type_sig: &EType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_tuple(
        type_sig: &types::TupleType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_tuple_from(
        type_sig: &Vec<EType>,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_vec(
        type_sig: &types::VecType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_vec_from(
        type_sig: &EType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_error() -> StaticType;

    fn build_range(
        type_sig: &types::RangeType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_range_from(
        type_sig: NumberType,
        inclusiv: bool,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_fn(
        params: &Vec<EType>,
        ret: &EType,
        scope_params_size: usize,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_closure(
        params: &Vec<EType>,
        ret: &EType,
        closed: bool,
        scope_params_size: usize,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_unit() -> StaticType;

    fn build_any() -> StaticType;

    fn build_addr(
        type_sig: &types::AddrType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_addr_from(
        type_sig: &EType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;

    fn build_map(
        type_sig: &types::MapType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_map_from(
        key: &EType,
        value: &EType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError>;
}

pub trait BuildUserType {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<UserType, SemanticError>;
}

pub trait BuildVar {
    fn build_var(id: &ID, type_sig: &EType) -> Var;
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
    pub is_loop: bool,
}

impl Default for ScopeState {
    fn default() -> Self {
        Self {
            is_closure: ClosureState::DEFAULT,
            is_loop: false,
        }
    }
}
