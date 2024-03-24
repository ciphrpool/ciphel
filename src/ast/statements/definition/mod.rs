use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::{types::Type, utils::strings::ID},
    semantic::{
        scope::{var_impl::Var, ScopeApi},
        AccessLevel, MutRc,
    },
};

use super::{declaration::TypedVar, scope::Scope};

pub mod definition_gencode;
pub mod definition_parse;
pub mod definition_resolve;
pub mod definition_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Definition<InnerScope: ScopeApi> {
    Type(TypeDef),
    Fn(FnDef<InnerScope>),
    Event(EventDef<InnerScope>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDef {
    Struct(StructDef),
    Union(UnionDef),
    Enum(EnumDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub id: ID,
    pub fields: Vec<(ID, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub id: ID,
    pub variants: Vec<(ID, Vec<(ID, Type)>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub id: ID,
    pub values: Vec<ID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef<InnerScope: ScopeApi> {
    id: ID,
    params: Vec<TypedVar>,
    ret: Box<Type>,
    scope: Scope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventDef<InnerScope: ScopeApi> {
    id: ID,
    condition: EventCondition,
    scope: Scope<InnerScope>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventCondition {}
