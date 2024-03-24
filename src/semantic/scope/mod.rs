use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
    slice::Iter,
};

use crate::{
    ast::{statements::definition, types, utils::strings::ID},
    vm::{allocator::stack::Offset, vm::CodeGenerationError},
};

use self::{
    chan_impl::Chan, event_impl::Event, static_types::StaticType, user_type_impl::UserType,
    var_impl::Var,
};

use super::{AccessLevel, EType, Either, MutRc, SemanticError};
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

    fn build_fn(
        params: &Vec<EType>,
        ret: &EType,
        scope: &Ref<Scope>,
    ) -> Result<StaticType, SemanticError>;
    fn build_closure(
        params: &Vec<EType>,
        ret: &EType,
        closed: bool,
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

pub trait BuildUserType<Scope: ScopeApi> {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<UserType, SemanticError>;
}

pub trait BuildVar<Scope: ScopeApi> {
    fn build_var(id: &ID, type_sig: &EType) -> Var;
}
pub trait BuildChan<Scope: ScopeApi> {
    fn build_chan(id: &ID, type_sig: &EType) -> Chan;
}
pub trait BuildEvent<Scope: ScopeApi> {
    fn build_event(scope: &Ref<Scope>, event: &definition::EventDef<Scope>) -> Event;
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

pub trait ScopeApi
where
    Self: Sized + Clone + Debug,
{
    fn spawn(parent: &MutRc<Self>, vars: Vec<Var>) -> Result<MutRc<Self>, SemanticError>;

    fn register_type(&mut self, id: &ID, reg: UserType) -> Result<(), SemanticError>;
    fn register_chan(&mut self, reg: &ID) -> Result<(), SemanticError>;
    fn register_var(&mut self, reg: Var) -> Result<(), SemanticError>;
    fn register_event(&mut self, reg: Event) -> Result<(), SemanticError>;

    fn state(&self) -> ScopeState;
    fn to_closure(&mut self, state: ClosureState);
    fn capture(&self, var: Rc<Var>) -> bool;
    fn to_generator(&mut self);
    fn to_loop(&mut self);

    fn find_var(&self, id: &ID) -> Result<Rc<Var>, SemanticError>;

    fn access_var(&self, id: &ID) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError>;
    fn access_var_in_parent(
        &self,
        id: &ID,
    ) -> Result<(Rc<Var>, Offset, AccessLevel), CodeGenerationError>;
    fn update_var_offset(&self, id: &ID, offset: Offset) -> Result<Rc<Var>, CodeGenerationError>;
    fn stack_top(&self) -> Option<usize>;
    fn update_stack_top(&self, top: usize) -> Result<(), CodeGenerationError>;

    fn env_vars(&self) -> Vec<Rc<Var>>;
    fn vars(&self) -> Iter<(Rc<Var>, Cell<Offset>)>;

    fn find_chan(&self) -> Result<&Chan, SemanticError>;
    fn find_type(&self, id: &ID) -> Result<Rc<UserType>, SemanticError>;
    fn find_event(&self) -> Result<&Event, SemanticError>;
}
