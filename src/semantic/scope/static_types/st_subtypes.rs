use crate::{
    e_static, e_user, p_num,
    semantic::{
        scope::{type_traits::GetSubTypes, user_type_impl::UserType},
        EType, Either, SizeOf,
    },
};

use super::{
    AddrType, NumberType, PrimitiveType, SliceType, StaticType, StrSliceType, StringType, TupleType,
};

impl GetSubTypes for StaticType {
    fn get_nth(&self, n: &usize) -> Option<EType> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::StaticFn(value) => value.params.get(*n).map(|v| v.clone()),
            StaticType::Closure(value) => value.params.get(*n).map(|v| v.clone()),
            StaticType::Tuple(value) => value.0.get(*n).map(|v| v.clone()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_nth(value, n),
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
            StaticType::StrSlice(_) => None,
            StaticType::Range(_) => None,
        }
    }

    fn get_item(&self) -> Option<EType> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(SliceType { size: _, item_type }) => Some(item_type.as_ref().clone()),
            StaticType::Vec(value) => Some(value.0.as_ref().clone()),
            StaticType::StaticFn(_) => None,
            StaticType::Closure(_) => None,
            StaticType::Tuple(_) => None,
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => Some(e_static!(Self::Error)),
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_item(value),
            StaticType::Map(value) => Some(value.values_type.as_ref().clone()),
            StaticType::String(_) => Some(e_static!(Self::Primitive(PrimitiveType::Char))),
            StaticType::StrSlice(_) => Some(e_static!(Self::Primitive(PrimitiveType::Char))),
            StaticType::Range(value) => Some(Either::Static(
                Self::Primitive(PrimitiveType::Number(value.num)).into(),
            )),
        }
    }

    fn get_key(&self) -> Option<EType> {
        match self {
            StaticType::Map(value) => Some(value.keys_type.as_ref().clone()),
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_key(value),
            StaticType::Slice(_) => Some(p_num!(U64)),
            StaticType::Vec(_) => Some(p_num!(U64)),
            StaticType::String(_) => Some(p_num!(U64)),
            StaticType::StrSlice(_) => Some(p_num!(U64)),
            _ => None,
        }
    }

    fn get_return(&self) -> Option<EType> {
        match self {
            StaticType::StaticFn(value) => Some(value.ret.as_ref().clone()),
            StaticType::Closure(value) => Some(value.ret.as_ref().clone()),
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_return(value),
            _ => None,
        }
    }
    fn get_fields(&self) -> Option<Vec<(Option<String>, EType)>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::StaticFn(value) => Some(
                value
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
            StaticType::Closure(value) => Some(
                value
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
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
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_fields(value),
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
            StaticType::StrSlice(_) => None,
            StaticType::Range(_) => None,
        }
    }

    fn get_length(&self) -> Option<usize> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(SliceType { size, item_type: _ }) => Some(size.clone()),
            StaticType::Vec(_) => None,
            StaticType::StaticFn(_) => None,
            StaticType::Closure(_) => None,
            StaticType::Tuple(value) => Some(value.0.len()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(AddrType(value)) => <EType as GetSubTypes>::get_length(value),
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
            StaticType::StrSlice(StrSliceType { size }) => Some(size.clone()),
            StaticType::Range(_) => None,
        }
    }

    fn get_inline_field_offset(&self, index: usize) -> Option<usize> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::StaticFn(_) => None,
            StaticType::Closure(_) => None,
            StaticType::Tuple(TupleType(fields)) => {
                Some(fields.iter().take(index).map(|field| field.size_of()).sum())
            }
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(_) => None,
            StaticType::Map(_) => None,
            StaticType::String(_) => None,
            StaticType::StrSlice(_) => None,
            StaticType::Range(_) => None,
        }
    }
}
