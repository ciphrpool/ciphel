use crate::semantic::SizeOf;

use super::{
    AddrType, ChanType, ClosureType, FnType, MapType, NumberType, PrimitiveType, RangeType,
    SliceType, StaticType, StrSliceType, StringType, TupleType, VecType,
};

impl SizeOf for StaticType {
    fn size_of(&self) -> usize {
        match self {
            StaticType::Primitive(value) => value.size_of(),
            StaticType::Slice(value) => value.size_of(),
            StaticType::Vec(value) => value.size_of(),
            StaticType::StaticFn(value) => value.size_of(),
            StaticType::Closure(value) => value.size_of(),
            StaticType::Chan(value) => value.size_of(),
            StaticType::Tuple(value) => value.size_of(),
            StaticType::Unit => 0,
            StaticType::Any => 0,
            StaticType::Error => todo!(),
            StaticType::Address(value) => value.size_of(),
            StaticType::Map(value) => value.size_of(),
            StaticType::String(value) => value.size_of(),
            StaticType::StrSlice(value) => value.size_of(),
            StaticType::Range(value) => value.size_of(),
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
        self.size * self.item_type.size_of()
    }
}

impl SizeOf for StringType {
    fn size_of(&self) -> usize {
        8
    }
}
impl SizeOf for StrSliceType {
    fn size_of(&self) -> usize {
        self.size
    }
}

impl SizeOf for RangeType {
    fn size_of(&self) -> usize {
        self.num.size_of() * 3
    }
}

impl SizeOf for VecType {
    fn size_of(&self) -> usize {
        8
    }
}

impl SizeOf for FnType {
    fn size_of(&self) -> usize {
        8
    }
}

impl SizeOf for ClosureType {
    fn size_of(&self) -> usize {
        8
    }
}

impl SizeOf for ChanType {
    fn size_of(&self) -> usize {
        8
    }
}

impl SizeOf for TupleType {
    fn size_of(&self) -> usize {
        self.0.iter().map(|sub| sub.size_of()).sum()
    }
}

impl SizeOf for AddrType {
    fn size_of(&self) -> usize {
        8
    }
}

impl SizeOf for MapType {
    fn size_of(&self) -> usize {
        8
    }
}
