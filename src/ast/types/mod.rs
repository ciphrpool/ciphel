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
    StrSlice(StrSliceType),
    Range(RangeType),
    UserType(ID),
    Vec(VecType),
    Function(FunctionType),
    Closure(ClosureType),
    Lambda(LambdaType),
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
    F64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliceType {
    pub size: usize,
    pub item_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrSliceType {}

#[derive(Debug, Clone, PartialEq)]
pub struct StringType();

#[derive(Debug, Clone, PartialEq)]
pub struct VecType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Types,
    pub ret: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureType {
    pub params: Types,
    pub ret: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaType {
    pub params: Types,
    pub ret: SubType,
}

pub type Types = Vec<Type>;

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType(pub Types);

#[derive(Debug, Clone, PartialEq)]
pub struct AddrType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct RangeType {
    pub num: NumberType,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub keys_type: SubType,
    pub values_type: SubType,
}
