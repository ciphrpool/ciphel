use crate::semantic::{
    scope::{type_traits::GetSubTypes, user_type_impl::UserType},
    Either, SizeOf,
};

use super::{
    AddrType, KeyType, NumberType, PrimitiveType, SliceType, StaticType, StringType, TupleType,
};

impl GetSubTypes for StaticType {
    fn get_nth(&self, n: &usize) -> Option<Either<UserType, StaticType>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::Fn(value) => value.params.get(*n).map(|v| v.clone()),
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => value.0.get(*n).map(|v| v.clone()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_nth(value, n)
            }
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
        }
    }

    fn get_item(&self) -> Option<Either<UserType, StaticType>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(SliceType { size, item_type }) => Some(item_type.as_ref().clone()),
            StaticType::Vec(value) => Some(value.0.as_ref().clone()),
            StaticType::Fn(_) => None,
            StaticType::Chan(value) => Some(value.0.as_ref().clone()),
            StaticType::Tuple(_) => None,
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => Some(Either::Static(Self::Error.into())),
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_item(value)
            }
            StaticType::Map(value) => Some(value.values_type.as_ref().clone()),
            StaticType::String(_) => {
                Some(Either::Static(Self::Primitive(PrimitiveType::Char).into()))
            }
        }
    }

    fn get_key(&self) -> Option<Either<UserType, StaticType>> {
        match self {
            StaticType::Map(value) => match &value.keys_type {
                KeyType::Primitive(value) => {
                    Some(Either::Static(StaticType::Primitive(value.clone()).into()))
                }
                KeyType::Address(value) => {
                    Some(Either::Static(StaticType::Address(value.clone()).into()))
                }
                KeyType::String(_) => Some(Either::Static(StaticType::String(StringType()).into())),
                KeyType::Enum(value) => Some(Either::User(UserType::Enum(value.clone()).into())),
            },
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_key(value)
            }
            StaticType::Slice(_) => Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
            StaticType::Vec(_) => Some(Either::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
            _ => None,
        }
    }

    fn get_return(&self) -> Option<Either<UserType, StaticType>> {
        match self {
            StaticType::Fn(value) => Some(value.ret.as_ref().clone()),
            StaticType::Chan(value) => Some(value.0.as_ref().clone()),
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_return(value)
            }
            _ => None,
        }
    }
    fn get_fields(&self) -> Option<Vec<(Option<String>, Either<UserType, StaticType>)>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::Fn(value) => Some(
                value
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => Some(
                value
                    .0
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_fields(value)
            }
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
        }
    }

    fn get_length(&self) -> Option<usize> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(SliceType { size, item_type }) => Some(size.clone()),
            StaticType::Vec(_) => None,
            StaticType::Fn(_) => None,
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => Some(value.0.len()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => {
                <Either<UserType, StaticType> as GetSubTypes>::get_length(value)
            }
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
        }
    }

    fn get_inline_field_offset(&self, index: usize) -> Option<usize> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::Fn(_) => None,
            StaticType::Chan(_) => None,
            StaticType::Tuple(TupleType(fields)) => Some(
                fields
                    .iter()
                    .take(index - 1)
                    .map(|field| field.size_of())
                    .sum(),
            ),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(_) => None,
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
        }
    }
}
