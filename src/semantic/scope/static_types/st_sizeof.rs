use crate::semantic::SizeOf;

use super::{
    AddrType, ChanType, FnType, MapType, PrimitiveType, SliceType, StaticType, TupleType, VecType,
};

impl SizeOf for StaticType {
    fn size_of(&self) -> usize {
        match self {
            StaticType::Primitive(value) => value.size_of(),
            StaticType::Slice(value) => value.size_of(),
            StaticType::Vec(value) => value.size_of(),
            StaticType::Fn(value) => value.size_of(),
            StaticType::Chan(value) => value.size_of(),
            StaticType::Tuple(value) => value.size_of(),
            StaticType::Unit => todo!(),
            StaticType::Any => todo!(),
            StaticType::Error => todo!(),
            StaticType::Address(value) => value.size_of(),
            StaticType::Map(value) => value.size_of(),
        }
    }
}

impl SizeOf for PrimitiveType {
    fn size_of(&self) -> usize {
        match self {
            PrimitiveType::Number => 8,
            PrimitiveType::Float => 8,
            PrimitiveType::Char => 1,
            PrimitiveType::Bool => 1,
        }
    }
}

impl SizeOf for SliceType {
    fn size_of(&self) -> usize {
        match self {
            SliceType::String => 8,
            SliceType::List(size, subtype) => size * subtype.size_of(),
        }
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
