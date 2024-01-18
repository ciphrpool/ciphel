use std::cell::Ref;
use std::rc::Rc;

use crate::semantic::{EitherType, SemanticError, TypeOf};



use super::{
    user_type_impl::{Enum, UserType}, ScopeApi,
};

pub mod st_builder;
pub mod st_compatible_with;
pub mod st_merging;
pub mod st_operand_merging;
pub mod st_sizeof;
pub mod st_subtypes;
pub mod st_type_checking;

type SubType = Box<EitherType<UserType, StaticType>>;

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
    Number,
    Float,
    Char,
    Bool,
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
pub type Types = Vec<EitherType<UserType, StaticType>>;

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

impl<Scope: ScopeApi<StaticType = StaticType>> TypeOf<Scope> for StaticType {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<EitherType<<Scope as ScopeApi>::UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        Ok(EitherType::Static(Rc::new(self.clone())))
    }
}
