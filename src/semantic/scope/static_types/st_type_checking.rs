use crate::semantic::{
    scope::{type_traits::TypeChecking, user_type_impl::UserType},
    EType, Either,
};

use super::{AddrType, NumberType, PrimitiveType, StaticType};

impl TypeChecking for StaticType {
    fn is_iterable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => true,
            StaticType::Vec(_) => true,
            StaticType::StaticFn(_) => false,
            StaticType::Closure(_) => false,
            StaticType::Chan(_) => true,
            StaticType::Tuple(_) => false,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_iterable(value),
            StaticType::Map(_) => true,
            StaticType::String(_) => true,
            StaticType::StrSlice(_) => true,
            StaticType::Range(_) => true,
        }
    }
    fn is_indexable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => true,
            StaticType::Vec(_) => true,
            StaticType::StaticFn(_) => false,
            StaticType::Closure(_) => false,
            StaticType::Chan(_) => false,
            StaticType::Tuple(_) => false,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_iterable(value),
            StaticType::Map(_) => false,
            StaticType::String(_) => true,
            StaticType::StrSlice(_) => true,
            StaticType::Range(_) => false,
        }
    }
    fn is_dotnum_indexable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => false,
            StaticType::Vec(_) => false,
            StaticType::StaticFn(_) => false,
            StaticType::Closure(_) => false,
            StaticType::Chan(_) => false,
            StaticType::Tuple(_) => true,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(AddrType(value)) => {
                <EType as TypeChecking>::is_dotnum_indexable(value)
            }
            StaticType::Map(_) => false,
            StaticType::String(_) => false,
            StaticType::StrSlice(_) => false,
            StaticType::Range(_) => true,
        }
    }
    fn is_channel(&self) -> bool {
        match self {
            StaticType::Chan(_) => true,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_channel(value),
            _ => false,
        }
    }
    fn is_boolean(&self) -> bool {
        match self {
            StaticType::Primitive(PrimitiveType::Bool) => true,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_boolean(value),
            _ => false,
        }
    }

    fn is_callable(&self) -> bool {
        match self {
            StaticType::StaticFn(_) => true,
            StaticType::Closure(_) => true,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_callable(value),
            _ => false,
        }
    }
    fn is_any(&self) -> bool {
        match self {
            StaticType::Any => true,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_any(value),
            _ => false,
        }
    }
    fn is_unit(&self) -> bool {
        match self {
            StaticType::Unit => true,
            StaticType::Address(AddrType(value)) => <EType as TypeChecking>::is_unit(value),
            _ => false,
        }
    }

    fn is_addr(&self) -> bool {
        match self {
            StaticType::Address(AddrType(_value)) => true,
            _ => false,
        }
    }

    fn is_u64(&self) -> bool {
        match self {
            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)) => true,
            _ => false,
        }
    }

    fn is_char(&self) -> bool {
        match self {
            StaticType::Primitive(PrimitiveType::Char) => true,
            _ => false,
        }
    }

    fn is_vec(&self) -> bool {
        match self {
            StaticType::Vec(_) => true,
            _ => false,
        }
    }
    fn is_map(&self) -> bool {
        match self {
            StaticType::Map(_) => true,
            _ => false,
        }
    }
}
