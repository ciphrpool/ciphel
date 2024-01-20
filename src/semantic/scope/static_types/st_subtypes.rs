use crate::semantic::{
    scope::{type_traits::GetSubTypes, user_type_impl::UserType, ScopeApi},
    EitherType,
};

use super::{AddrType, KeyType, NumberType, PrimitiveType, SliceType, StaticType};

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> GetSubTypes<Scope> for StaticType {
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
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
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_nth(value, n)
            }
            StaticType::Map(_) => None,
        }
    }

    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(value) => match value {
                SliceType::String => Some(EitherType::Static(
                    Self::Primitive(PrimitiveType::Char).into(),
                )),
                SliceType::List(_, value) => Some(value.as_ref().clone()),
            },
            StaticType::Vec(value) => Some(value.0.as_ref().clone()),
            StaticType::Fn(_) => None,
            StaticType::Chan(value) => Some(value.0.as_ref().clone()),
            StaticType::Tuple(_) => None,
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => Some(EitherType::Static(Self::Error.into())),
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_item(value)
            }
            StaticType::Map(value) => Some(value.values_type.as_ref().clone()),
        }
    }

    fn get_key(
        &self,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            StaticType::Map(value) => match &value.keys_type {
                KeyType::Primitive(value) => Some(EitherType::Static(
                    StaticType::Primitive(value.clone()).into(),
                )),
                KeyType::Address(value) => Some(EitherType::Static(
                    StaticType::Address(value.clone()).into(),
                )),
                KeyType::Slice(value) => {
                    Some(EitherType::Static(StaticType::Slice(value.clone()).into()))
                }
                KeyType::Enum(value) => {
                    Some(EitherType::User(UserType::Enum(value.clone()).into()))
                }
            },
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_key(value)
            }
            StaticType::Slice(_) => Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
            StaticType::Vec(_) => Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
            )),
            _ => None,
        }
    }

    fn get_return(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            StaticType::Fn(value) => Some(value.ret.as_ref().clone()),
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_return(value)
            }
            _ => None,
        }
    }
    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
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
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_fields(value)
            }
            StaticType::Map(_) => None,
        }
    }

    fn get_length(&self) -> Option<usize> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(value) => match value {
                SliceType::String => None,
                SliceType::List(size, _) => Some(size.clone()),
            },
            StaticType::Vec(_) => None,
            StaticType::Fn(_) => None,
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => Some(value.0.len()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => {
                <EitherType<UserType, StaticType> as GetSubTypes<Scope>>::get_length(value)
            }
            StaticType::Map(_) => None,
        }
    }
}
