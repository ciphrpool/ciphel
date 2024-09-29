use crate::semantic::{CompatibleWith, EType, MergeType, SemanticError, SizeOf};

type SubType = Box<EType>;

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Primitive(PrimitiveType),
    Slice(SliceType),
    String(StringType),
    StrSlice(StrSliceType),
    Vec(VecType),
    Closure(ClosureType),
    Lambda(LambdaType),
    Function(FunctionType),
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

pub type Types = Vec<EType>;

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType(pub Types);

#[derive(Debug, Clone, PartialEq)]
pub struct AddrType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub keys_type: SubType,
    pub values_type: SubType,
}

pub const POINTER_SIZE: usize = 8;

impl SizeOf for StaticType {
    fn size_of(&self) -> usize {
        match self {
            StaticType::Primitive(value) => value.size_of(),
            StaticType::Slice(value) => value.size_of(),
            StaticType::Vec(value) => value.size_of(),
            StaticType::Function(value) => value.size_of(),
            StaticType::Closure(value) => value.size_of(),
            StaticType::Tuple(value) => value.size_of(),
            StaticType::Unit => 0,
            StaticType::Any => 0,
            StaticType::Error => 1,
            StaticType::Address(value) => value.size_of(),
            StaticType::Map(value) => value.size_of(),
            StaticType::String(value) => value.size_of(),
            StaticType::StrSlice(value) => value.size_of(),
            StaticType::Lambda(value) => value.size_of(),
        }
    }
}

impl SizeOf for NumberType {
    fn size_of(&self) -> usize {
        match self {
            super::NumberType::U8 => 1,
            super::NumberType::U16 => 2,
            super::NumberType::U32 => 4,
            super::NumberType::U64 => 8,
            super::NumberType::U128 => 16,
            super::NumberType::I8 => 1,
            super::NumberType::I16 => 2,
            super::NumberType::I32 => 4,
            super::NumberType::I64 => 8,
            super::NumberType::I128 => 16,
            super::NumberType::F64 => 8,
        }
    }
}
impl SizeOf for PrimitiveType {
    fn size_of(&self) -> usize {
        match self {
            PrimitiveType::Number(num) => num.size_of(),
            PrimitiveType::Char => 4,
            PrimitiveType::Bool => 1,
        }
    }
}

impl SizeOf for SliceType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for StringType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}
impl SizeOf for StrSliceType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for VecType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for FunctionType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for ClosureType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for LambdaType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for TupleType {
    fn size_of(&self) -> usize {
        self.0.iter().map(|sub| sub.size_of()).sum()
    }
}

impl SizeOf for AddrType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
}

impl SizeOf for MapType {
    fn size_of(&self) -> usize {
        POINTER_SIZE
    }
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
            (StaticType::String(x), StaticType::String(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::StrSlice(x), StaticType::StrSlice(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Closure(x), StaticType::Closure(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Lambda(x), StaticType::Lambda(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (StaticType::Function(x), StaticType::Function(y)) => {
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
