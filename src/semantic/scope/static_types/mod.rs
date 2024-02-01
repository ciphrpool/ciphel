use std::cell::Ref;
use std::rc::Rc;

use crate::semantic::{Either, SemanticError, TypeOf};

use super::{
    user_type_impl::{Enum, UserType},
    ScopeApi,
};

pub mod st_builder;
pub mod st_compatible_with;
pub mod st_deserialize;
pub mod st_merging;
pub mod st_operand_merging;
pub mod st_sizeof;
pub mod st_subtypes;
pub mod st_type_checking;

type SubType = Box<Either<UserType, StaticType>>;

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Primitive(PrimitiveType),
    Slice(SliceType),
    Vec(VecType),
    Fn(FnType),
    Chan(ChanType),
    Tuple(TupleType),
    Unit,
    Any,
    Error,
    Address(AddrType),
    Map(MapType),
}
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Number(NumberType),
    Float,
    Char,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum NumberType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SliceType {
    String,
    List(usize, SubType),
}
#[derive(Debug, Clone, PartialEq)]
pub struct VecType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct FnType {
    pub params: Types,
    pub ret: SubType,
}
pub type Types = Vec<Either<UserType, StaticType>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ChanType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType(pub Types);

#[derive(Debug, Clone, PartialEq)]
pub struct AddrType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub keys_type: KeyType,
    pub values_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    Primitive(PrimitiveType),
    Address(AddrType),
    Slice(SliceType),
    Enum(Enum),
}

impl<Scope: ScopeApi> TypeOf<Scope> for StaticType {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        Ok(Either::Static(Rc::new(self.clone())))
    }
}

impl NumberType {
    pub fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (NumberType::U8, NumberType::U8) => NumberType::U8,
            (NumberType::U8, NumberType::U16) => NumberType::U16,
            (NumberType::U8, NumberType::U32) => NumberType::U32,
            (NumberType::U8, NumberType::U64) => NumberType::U64,
            (NumberType::U8, NumberType::U128) => NumberType::U128,
            (NumberType::U8, NumberType::I8) => NumberType::I16,
            (NumberType::U8, NumberType::I16) => NumberType::I16,
            (NumberType::U8, NumberType::I32) => NumberType::I32,
            (NumberType::U8, NumberType::I64) => NumberType::I64,
            (NumberType::U8, NumberType::I128) => NumberType::I128,
            (NumberType::U16, NumberType::U8) => NumberType::U16,
            (NumberType::U16, NumberType::U16) => NumberType::U16,
            (NumberType::U16, NumberType::U32) => NumberType::U32,
            (NumberType::U16, NumberType::U64) => NumberType::U64,
            (NumberType::U16, NumberType::U128) => NumberType::U128,
            (NumberType::U16, NumberType::I8) => NumberType::I32,
            (NumberType::U16, NumberType::I16) => NumberType::I32,
            (NumberType::U16, NumberType::I32) => NumberType::I32,
            (NumberType::U16, NumberType::I64) => NumberType::I64,
            (NumberType::U16, NumberType::I128) => NumberType::I128,
            (NumberType::U32, NumberType::U8) => NumberType::U32,
            (NumberType::U32, NumberType::U16) => NumberType::U32,
            (NumberType::U32, NumberType::U32) => NumberType::U32,
            (NumberType::U32, NumberType::U64) => NumberType::U64,
            (NumberType::U32, NumberType::U128) => NumberType::U128,
            (NumberType::U32, NumberType::I8) => NumberType::I64,
            (NumberType::U32, NumberType::I16) => NumberType::I64,
            (NumberType::U32, NumberType::I32) => NumberType::I64,
            (NumberType::U32, NumberType::I64) => NumberType::I64,
            (NumberType::U32, NumberType::I128) => NumberType::I128,
            (NumberType::U64, NumberType::U8) => NumberType::U64,
            (NumberType::U64, NumberType::U16) => NumberType::U64,
            (NumberType::U64, NumberType::U32) => NumberType::U64,
            (NumberType::U64, NumberType::U64) => NumberType::U64,
            (NumberType::U64, NumberType::U128) => NumberType::U128,
            (NumberType::U64, NumberType::I8) => NumberType::I128,
            (NumberType::U64, NumberType::I16) => NumberType::I128,
            (NumberType::U64, NumberType::I32) => NumberType::I128,
            (NumberType::U64, NumberType::I64) => NumberType::I128,
            (NumberType::U64, NumberType::I128) => NumberType::I128,
            (NumberType::U128, NumberType::U8) => NumberType::U128,
            (NumberType::U128, NumberType::U16) => NumberType::U128,
            (NumberType::U128, NumberType::U32) => NumberType::U128,
            (NumberType::U128, NumberType::U64) => NumberType::U128,
            (NumberType::U128, NumberType::U128) => NumberType::U128,
            (NumberType::U128, NumberType::I8) => NumberType::I128,
            (NumberType::U128, NumberType::I16) => NumberType::I128,
            (NumberType::U128, NumberType::I32) => NumberType::I128,
            (NumberType::U128, NumberType::I64) => NumberType::I128,
            (NumberType::U128, NumberType::I128) => NumberType::I128,
            (NumberType::I8, NumberType::U8) => NumberType::I16,
            (NumberType::I8, NumberType::U16) => NumberType::I32,
            (NumberType::I8, NumberType::U32) => NumberType::I64,
            (NumberType::I8, NumberType::U64) => NumberType::I128,
            (NumberType::I8, NumberType::U128) => NumberType::I128,
            (NumberType::I8, NumberType::I8) => NumberType::I8,
            (NumberType::I8, NumberType::I16) => NumberType::I16,
            (NumberType::I8, NumberType::I32) => NumberType::I32,
            (NumberType::I8, NumberType::I64) => NumberType::I64,
            (NumberType::I8, NumberType::I128) => NumberType::I128,
            (NumberType::I16, NumberType::U8) => NumberType::I16,
            (NumberType::I16, NumberType::U16) => NumberType::I32,
            (NumberType::I16, NumberType::U32) => NumberType::I64,
            (NumberType::I16, NumberType::U64) => NumberType::I128,
            (NumberType::I16, NumberType::U128) => NumberType::I128,
            (NumberType::I16, NumberType::I8) => NumberType::I16,
            (NumberType::I16, NumberType::I16) => NumberType::I16,
            (NumberType::I16, NumberType::I32) => NumberType::I32,
            (NumberType::I16, NumberType::I64) => NumberType::I64,
            (NumberType::I16, NumberType::I128) => NumberType::I128,
            (NumberType::I32, NumberType::U8) => NumberType::I32,
            (NumberType::I32, NumberType::U16) => NumberType::I32,
            (NumberType::I32, NumberType::U32) => NumberType::I64,
            (NumberType::I32, NumberType::U64) => NumberType::I64,
            (NumberType::I32, NumberType::U128) => NumberType::I128,
            (NumberType::I32, NumberType::I8) => NumberType::I32,
            (NumberType::I32, NumberType::I16) => NumberType::I32,
            (NumberType::I32, NumberType::I32) => NumberType::I32,
            (NumberType::I32, NumberType::I64) => NumberType::I64,
            (NumberType::I32, NumberType::I128) => NumberType::I128,
            (NumberType::I64, NumberType::U8) => NumberType::I64,
            (NumberType::I64, NumberType::U16) => NumberType::I64,
            (NumberType::I64, NumberType::U32) => NumberType::I64,
            (NumberType::I64, NumberType::U64) => NumberType::I128,
            (NumberType::I64, NumberType::U128) => NumberType::I128,
            (NumberType::I64, NumberType::I8) => NumberType::I64,
            (NumberType::I64, NumberType::I16) => NumberType::I64,
            (NumberType::I64, NumberType::I32) => NumberType::I64,
            (NumberType::I64, NumberType::I64) => NumberType::I64,
            (NumberType::I64, NumberType::I128) => NumberType::I128,
            (NumberType::I128, NumberType::U8) => NumberType::I128,
            (NumberType::I128, NumberType::U16) => NumberType::I128,
            (NumberType::I128, NumberType::U32) => NumberType::I128,
            (NumberType::I128, NumberType::U64) => NumberType::I128,
            (NumberType::I128, NumberType::U128) => NumberType::I128,
            (NumberType::I128, NumberType::I8) => NumberType::I128,
            (NumberType::I128, NumberType::I16) => NumberType::I128,
            (NumberType::I128, NumberType::I32) => NumberType::I128,
            (NumberType::I128, NumberType::I64) => NumberType::I128,
            (NumberType::I128, NumberType::I128) => NumberType::I128,
        }
    }
}
