use std::cell::Ref;

use crate::{
    ast,
    semantic::{
        scope::{user_type_impl::UserType, BuildStaticType},
        EType, Either, SemanticError, TypeOf,
    },
};

use super::{
    AddrType, ClosureType, FnType, MapType, NumberType, PrimitiveType, RangeType, SliceType,
    StaticType, TupleType, VecType,
};
use crate::semantic::scope::scope::Scope;

impl BuildStaticType for StaticType {
    fn build_primitive(
        type_sig: &ast::types::PrimitiveType,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Primitive(match type_sig {
            ast::types::PrimitiveType::Number(ast::types::NumberType::U8) => {
                PrimitiveType::Number(NumberType::U8)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::U16) => {
                PrimitiveType::Number(NumberType::U16)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::U32) => {
                PrimitiveType::Number(NumberType::U32)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::U64) => {
                PrimitiveType::Number(NumberType::U64)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::U128) => {
                PrimitiveType::Number(NumberType::U128)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::I8) => {
                PrimitiveType::Number(NumberType::I8)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::I16) => {
                PrimitiveType::Number(NumberType::I16)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::I32) => {
                PrimitiveType::Number(NumberType::I32)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::I64) => {
                PrimitiveType::Number(NumberType::I64)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::I128) => {
                PrimitiveType::Number(NumberType::I128)
            }
            ast::types::PrimitiveType::Number(ast::types::NumberType::F64) => {
                PrimitiveType::Number(NumberType::F64)
            }
            ast::types::PrimitiveType::Char => PrimitiveType::Char,
            ast::types::PrimitiveType::Bool => PrimitiveType::Bool,
        }))
    }

    fn build_str_slice(
        type_sig: &ast::types::StrSliceType,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::StrSlice(super::StrSliceType {
            size: type_sig.size,
        }))
    }

    fn build_string(
        _type_sig: &ast::types::StringType,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::String(super::StringType()))
    }

    fn build_slice(
        type_sig: &ast::types::SliceType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let inner = type_sig.item_type.type_of(&scope)?;
        Ok(Self::Slice(SliceType {
            size: type_sig.size.clone(),
            item_type: Box::new(inner),
        }))
    }

    fn build_slice_from(
        size: &usize,
        type_sig: &EType,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Slice(SliceType {
            size: size.clone(),
            item_type: Box::new(type_sig.clone()),
        }))
    }

    fn build_tuple(
        type_sig: &ast::types::TupleType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut vec = Vec::with_capacity(type_sig.0.len());
        for subtype in &type_sig.0 {
            let subtype = subtype.type_of(&scope)?;
            vec.push(subtype);
        }
        Ok(Self::Tuple(TupleType(vec)))
    }

    fn build_tuple_from(
        type_sig: &Vec<Either>,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut vec = Vec::with_capacity(type_sig.len());
        for subtype in type_sig {
            let subtype = subtype.type_of(&scope)?;
            vec.push(subtype);
        }
        Ok(Self::Tuple(TupleType(vec)))
    }

    fn build_vec(
        type_sig: &ast::types::VecType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(&scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_vec_from(
        type_sig: &Either,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_error() -> Self {
        Self::Error
    }

    fn build_fn(
        params: &Vec<EType>,
        ret: &EType,
        scope_params_size: usize,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::StaticFn(FnType {
            params: params.clone(),
            ret: Box::new(ret.clone()),
            scope_params_size,
        }))
    }

    fn build_closure(
        params: &Vec<Either>,
        ret: &Either,
        closed: bool,
        scope_params_size: usize,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Closure(ClosureType {
            params: params.clone(),
            ret: Box::new(ret.clone()),
            closed,
            scope_params_size,
        }))
    }

    fn build_unit() -> Self {
        Self::Unit
    }

    fn build_any() -> Self {
        Self::Any
    }

    fn build_addr(
        type_sig: &ast::types::AddrType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(&scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_addr_from(
        type_sig: &Either,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_map(
        type_sig: &ast::types::MapType,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        let key_type = type_sig.keys_type.type_of(&scope)?;
        let subtype = type_sig.values_type.type_of(&scope)?;
        Ok(Self::Map(MapType {
            keys_type: Box::new(key_type),
            values_type: Box::new(subtype),
        }))
    }

    fn build_map_from(
        key: &Either,
        value: &Either,
        scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Map(MapType {
            keys_type: Box::new(key.clone()),
            values_type: Box::new(value.clone()),
        }))
    }

    fn build_range(
        type_sig: &ast::types::RangeType,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<StaticType, SemanticError> {
        Ok(Self::Range(RangeType {
            num: match type_sig.num {
                ast::types::NumberType::U8 => NumberType::U8,
                ast::types::NumberType::U16 => NumberType::U16,
                ast::types::NumberType::U32 => NumberType::U32,
                ast::types::NumberType::U64 => NumberType::U64,
                ast::types::NumberType::U128 => NumberType::U128,
                ast::types::NumberType::I8 => NumberType::I8,
                ast::types::NumberType::I16 => NumberType::I16,
                ast::types::NumberType::I32 => NumberType::I32,
                ast::types::NumberType::I64 => NumberType::I64,
                ast::types::NumberType::I128 => NumberType::I128,
                ast::types::NumberType::F64 => NumberType::F64,
            },
            inclusive: type_sig.inclusive,
        }))
    }

    fn build_range_from(
        type_sig: NumberType,
        inclusive: bool,
        _scope: &std::sync::RwLockReadGuard<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Range(RangeType {
            num: type_sig,
            inclusive,
        }))
    }
}
