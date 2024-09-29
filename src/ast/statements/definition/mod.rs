use crate::ast::{types::Type, utils::strings::ID};

use super::{block::FunctionBlock, declaration::TypedVar};

pub mod definition_gencode;
pub mod definition_parse;
pub mod definition_resolve;
pub mod definition_typeof;

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Type(TypeDef),
    Fn(FnDef),
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
    name: String,
    id: Option<u64>,
    params: Vec<TypedVar>,
    ret: Box<Type>,
    scope: FunctionBlock,
}
