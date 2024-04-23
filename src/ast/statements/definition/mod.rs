

use crate::{
    ast::{types::Type, utils::strings::ID},
};

use super::{block::Block, declaration::TypedVar};

pub mod definition_gencode;
pub mod definition_parse;
pub mod definition_resolve;
pub mod definition_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Type(TypeDef),
    Fn(FnDef),
    Event(EventDef),
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
pub struct FnDef {
    id: ID,
    params: Vec<TypedVar>,
    ret: Box<Type>,
    scope: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventDef {
    id: ID,
    condition: EventCondition,
    scope: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventCondition {}
