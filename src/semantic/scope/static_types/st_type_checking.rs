use crate::semantic::{
    scope::{type_traits::TypeChecking, user_type_impl::UserType, ScopeApi},
    EitherType,
};

use super::{AddrType, PrimitiveType, StaticType};

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> TypeChecking<Scope> for StaticType {
    fn is_iterable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => true,
            StaticType::Vec(_) => true,
            StaticType::Fn(_) => false,
            StaticType::Chan(_) => true,
            StaticType::Tuple(_) => false,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_iterable(value)
            }
            StaticType::Map(_) => true,
        }
    }
    fn is_indexable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => false,
            StaticType::Vec(_) => false,
            StaticType::Fn(_) => false,
            StaticType::Chan(_) => false,
            StaticType::Tuple(_) => true,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_indexable(value)
            }
            StaticType::Map(_) => false,
        }
    }
    fn is_channel(&self) -> bool {
        match self {
            StaticType::Chan(_) => true,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_channel(value)
            }
            _ => false,
        }
    }
    fn is_boolean(&self) -> bool {
        match self {
            StaticType::Primitive(PrimitiveType::Bool) => true,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_boolean(value)
            }
            _ => false,
        }
    }

    fn is_callable(&self) -> bool {
        match self {
            StaticType::Fn(_) => true,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_callable(value)
            }
            _ => false,
        }
    }
    fn is_any(&self) -> bool {
        match self {
            StaticType::Any => true,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_any(value)
            }
            _ => false,
        }
    }
    fn is_unit(&self) -> bool {
        match self {
            StaticType::Unit => true,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as TypeChecking<Scope>>::is_unit(value)
            }
            _ => false,
        }
    }

    fn is_addr(&self) -> bool {
        match self {
            StaticType::Address(AddrType(_value)) => true,
            _ => false,
        }
    }

    fn is_num(&self) -> bool {
        match self {
            StaticType::Primitive(PrimitiveType::Number(_)) => true,
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
