use std::cell::Ref;

use crate::{
    ast,
    semantic::{
        scope::{
            chan_impl::Chan, event_impl::Event, user_type_impl::UserType, var_impl::Var,
            BuildStaticType, ScopeApi,
        },
        Either, SemanticError, TypeOf,
    },
};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, NumberType, PrimitiveType, SliceType, StaticType,
    TupleType, VecType,
};

impl<Scope: ScopeApi> BuildStaticType<Scope> for StaticType {
    fn build_primitive(
        type_sig: &ast::types::PrimitiveType,
        _scope: &Ref<Scope>,
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
            ast::types::PrimitiveType::Float => PrimitiveType::Float,
            ast::types::PrimitiveType::Char => PrimitiveType::Char,
            ast::types::PrimitiveType::Bool => PrimitiveType::Bool,
        }))
    }

    fn build_slice(
        type_sig: &ast::types::SliceType,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        match type_sig {
            ast::types::SliceType::String => Ok(Self::Slice(SliceType::String)),
            ast::types::SliceType::List(size, inner) => {
                let inner = inner.type_of(&scope)?;
                Ok(Self::Slice(SliceType::List(size.clone(), Box::new(inner))))
            }
        }
    }

    fn build_slice_from(
        size: &usize,
        type_sig: &Either<UserType, StaticType>,
        _scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Slice(SliceType::List(
            size.clone(),
            Box::new(type_sig.clone()),
        )))
    }

    fn build_tuple(
        type_sig: &ast::types::TupleType,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut vec = Vec::with_capacity(type_sig.0.len());
        for subtype in &type_sig.0 {
            let subtype = subtype.type_of(&scope)?;
            vec.push(subtype);
        }
        Ok(Self::Tuple(TupleType(vec)))
    }

    fn build_tuple_from(
        type_sig: &Vec<Either<UserType, Self>>,
        scope: &Ref<Scope>,
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
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(&scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_vec_from(
        type_sig: &Either<UserType, Self>,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_error() -> Self {
        Self::Error
    }

    fn build_fn(type_sig: &ast::types::FnType, scope: &Ref<Scope>) -> Result<Self, SemanticError> {
        let mut params = Vec::with_capacity(type_sig.params.len());
        for subtype in &type_sig.params {
            let subtype = subtype.type_of(&scope)?;
            params.push(subtype);
        }
        let ret_type = type_sig.ret.type_of(&scope)?;
        Ok(Self::Fn(FnType {
            params,
            ret: Box::new(ret_type),
        }))
    }

    fn build_fn_from(
        params: &Vec<Either<UserType, Self>>,
        ret: &Either<UserType, Self>,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut out_params = Vec::with_capacity(params.len());
        for subtype in params {
            let subtype = subtype.type_of(&scope)?;
            out_params.push(subtype);
        }
        let ret_type = ret.type_of(&scope)?;
        Ok(Self::Fn(FnType {
            params: out_params,
            ret: Box::new(ret_type),
        }))
    }

    fn build_chan(
        type_sig: &ast::types::ChanType,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(&scope)?;
        Ok(Self::Chan(ChanType(Box::new(subtype))))
    }

    fn build_chan_from(
        type_sig: &Either<UserType, Self>,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Chan(ChanType(Box::new(subtype))))
    }

    fn build_unit() -> Self {
        Self::Unit
    }

    fn build_any() -> Self {
        Self::Any
    }

    fn build_addr(
        type_sig: &ast::types::AddrType,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(&scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_addr_from(
        type_sig: &Either<UserType, Self>,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_map(
        type_sig: &ast::types::MapType,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let key_type = {
            match &type_sig.keys_type {
                ast::types::KeyType::Primitive(value) => KeyType::Primitive(match value {
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
                    ast::types::PrimitiveType::Float => PrimitiveType::Float,
                    ast::types::PrimitiveType::Char => PrimitiveType::Char,
                    ast::types::PrimitiveType::Bool => PrimitiveType::Bool,
                }),
                ast::types::KeyType::Address(value) => {
                    let subtype = value.0.type_of(&scope)?;
                    KeyType::Address(AddrType(Box::new(subtype)))
                }
                ast::types::KeyType::Slice(value) => match value {
                    ast::types::SliceType::String => KeyType::Slice(SliceType::String),
                    ast::types::SliceType::List(size, inner) => {
                        let inner = inner.type_of(&scope)?;
                        KeyType::Slice(SliceType::List(size.clone(), Box::new(inner)))
                    }
                },
                ast::types::KeyType::EnumID(value) => {
                    let binding = scope.find_type(&value)?;
                    let UserType::Enum(enum_type) = binding.as_ref() else {
                        return Err(SemanticError::IncompatibleTypes);
                    };
                    KeyType::Enum(enum_type.clone())
                }
            }
        };

        let subtype = type_sig.type_of(&scope)?;
        Ok(Self::Map(MapType {
            keys_type: key_type,
            values_type: Box::new(subtype),
        }))
    }

    fn build_map_from(
        key: &Either<UserType, Self>,
        value: &Either<UserType, Self>,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let key_type = {
            match key {
                Either::Static(value) => match value.as_ref() {
                    StaticType::Primitive(value) => KeyType::Primitive(value.clone()),
                    StaticType::Slice(value) => KeyType::Slice(value.clone()),
                    StaticType::Address(value) => KeyType::Address(value.clone()),
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
                Either::User(value) => match value.as_ref() {
                    UserType::Enum(value) => KeyType::Enum(value.clone()),
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
            }
        };

        let subtype = value.type_of(&scope)?;
        Ok(Self::Map(MapType {
            keys_type: key_type,
            values_type: Box::new(subtype),
        }))
    }
}
