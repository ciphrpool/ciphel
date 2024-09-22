use std::sync::Arc;

use crate::semantic::{CompatibleWith, EType, MergeType, SemanticError, TypeOf};

pub mod st_deserialize;
pub mod st_merging;
pub mod st_sizeof;

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
pub struct StrSliceType();

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

impl MergeType for StaticType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError> {
        if self != other {
            return Err(SemanticError::IncompatibleTypes);
        }
        return Ok(EType::Static(self.clone()));
    }
}

impl CompatibleWith for StaticType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        match (self, other) {
            (StaticType::Primitive(x), StaticType::Primitive(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Slice(x), StaticType::Slice(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Map(x), StaticType::Map(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Vec(x), StaticType::Vec(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::Range(x), StaticType::Range(y)) => {
                (x == y).then(|| ()).ok_or(SemanticError::IncompatibleTypes)
            }
            (StaticType::String(x), StaticType::String(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::StrSlice(x), StaticType::StrSlice(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Closure(x), StaticType::Closure(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::StaticFn(x), StaticType::StaticFn(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Tuple(x), StaticType::Tuple(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Unit, StaticType::Unit) => Ok(()),
            (StaticType::Any, _) => Ok(()),
            (StaticType::Error, _) => Ok(()),
            (StaticType::Address(x), StaticType::Address(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}
