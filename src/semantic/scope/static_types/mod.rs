use std::cell::Ref;
use std::rc::Rc;

use crate::semantic::{EType, Either, SemanticError, TypeOf};

use super::user_type_impl::Enum;

pub mod st_builder;
pub mod st_compatible_with;
pub mod st_deserialize;
pub mod st_merging;
pub mod st_next;
pub mod st_operand_merging;
pub mod st_sizeof;
pub mod st_subtypes;
pub mod st_type_checking;
use crate::semantic::scope::scope::Scope;

type SubType = Box<EType>;

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Primitive(PrimitiveType),
    Slice(SliceType),
    String(StringType),
    StrSlice(StrSliceType),
    Vec(VecType),
    Closure(ClosureType),
    StaticFn(FnType),
    Range(RangeType),
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
    F64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliceType {
    pub size: usize,
    pub item_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrSliceType {
    pub size: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct StringType();

#[derive(Debug, Clone, PartialEq)]
pub struct VecType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct FnType {
    pub params: Types,
    pub ret: SubType,
    pub scope_params_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureType {
    pub params: Types,
    pub ret: SubType,
    pub closed: bool,
    pub scope_params_size: usize,
}

pub type Types = Vec<EType>;

#[derive(Debug, Clone, PartialEq)]
pub struct RangeType {
    pub num: NumberType,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType(pub Types);

#[derive(Debug, Clone, PartialEq)]
pub struct AddrType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub keys_type: SubType,
    pub values_type: SubType,
}

impl TypeOf for StaticType {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(Either::Static(Rc::new(self.clone())))
    }
}
