use super::utils::strings::ID;

pub mod types_parse;
pub mod types_resolve;
pub mod types_typeof;

type SubType = Box<Type>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Slice(SliceType),
    String(StringType),
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
    Number(NumberType),
    Float,
    Char,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
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
pub struct SliceType {
    pub size: usize,
    pub item_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringType();

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
    String(StringType),
    EnumID(ID),
}
