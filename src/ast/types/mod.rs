use super::utils::strings::ID;

pub mod types_parse;
pub mod types_resolve;
pub mod types_typeof;

type SubType = Box<Type>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Slice(SliceType),
    UserType(ID),
    Vec(VecType),
    Fn(FnType),
    Chan(ChanType),
    Tuple(TupleType),
    Unit,
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
pub type Types = Vec<Type>;

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
    EnumID(ID),
}
