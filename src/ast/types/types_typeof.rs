use super::{
    AddrType, ClosureType, MapType, PrimitiveType, RangeType, SliceType, StrSliceType, StringType,
    TupleType, Type, VecType,
};
use crate::e_static;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{self, StaticType};

use crate::semantic::SizeOf;

use crate::semantic::{CompatibleWith, EType};
use crate::semantic::{SemanticError, TypeOf};

impl TypeOf for Type {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Type::Primitive(value) => value.type_of(&scope_manager, scope_id),
            Type::Slice(value) => value.type_of(&scope_manager, scope_id),
            Type::StrSlice(value) => value.type_of(&scope_manager, scope_id),
            Type::UserType(value) => {
                let user_type = scope_manager.find_type_by_name(&value, scope_id)?;
                Ok(EType::User {
                    id: user_type.id,
                    size: user_type.def.size_of(),
                })
            }
            Type::Vec(value) => value.type_of(&scope_manager, scope_id),
            Type::Closure(value) => value.type_of(&scope_manager, scope_id),
            Type::Tuple(value) => value.type_of(&scope_manager, scope_id),
            Type::Unit => {
                let static_type = StaticType::Unit;
                let static_type = e_static!(static_type);
                Ok(static_type)
            }
            Type::Any => {
                let static_type = e_static!(StaticType::Any);
                Ok(static_type)
            }
            Type::Address(value) => value.type_of(&scope_manager, scope_id),
            Type::Map(value) => value.type_of(&scope_manager, scope_id),
            Type::String(value) => value.type_of(&scope_manager, scope_id),
            Type::Range(value) => value.type_of(&scope_manager, scope_id),
            Type::Error => Ok(e_static!(StaticType::Error)),
        }
    }
}
impl TypeOf for PrimitiveType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type = StaticType::Primitive(match self {
            PrimitiveType::Number(super::NumberType::U8) => {
                static_types::PrimitiveType::Number(static_types::NumberType::U8)
            }
            PrimitiveType::Number(super::NumberType::U16) => {
                static_types::PrimitiveType::Number(static_types::NumberType::U16)
            }
            PrimitiveType::Number(super::NumberType::U32) => {
                static_types::PrimitiveType::Number(static_types::NumberType::U32)
            }
            PrimitiveType::Number(super::NumberType::U64) => {
                static_types::PrimitiveType::Number(static_types::NumberType::U64)
            }
            PrimitiveType::Number(super::NumberType::U128) => {
                static_types::PrimitiveType::Number(static_types::NumberType::U128)
            }
            PrimitiveType::Number(super::NumberType::I8) => {
                static_types::PrimitiveType::Number(static_types::NumberType::I8)
            }
            PrimitiveType::Number(super::NumberType::I16) => {
                static_types::PrimitiveType::Number(static_types::NumberType::I16)
            }
            PrimitiveType::Number(super::NumberType::I32) => {
                static_types::PrimitiveType::Number(static_types::NumberType::I32)
            }
            PrimitiveType::Number(super::NumberType::I64) => {
                static_types::PrimitiveType::Number(static_types::NumberType::I64)
            }
            PrimitiveType::Number(super::NumberType::I128) => {
                static_types::PrimitiveType::Number(static_types::NumberType::I128)
            }
            PrimitiveType::Number(super::NumberType::F64) => {
                static_types::PrimitiveType::Number(static_types::NumberType::F64)
            }
            PrimitiveType::Char => static_types::PrimitiveType::Char,
            PrimitiveType::Bool => static_types::PrimitiveType::Bool,
        });
        Ok(EType::Static(static_type))
    }
}
impl CompatibleWith for static_types::PrimitiveType {}

impl TypeOf for SliceType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(StaticType::Slice(static_types::SliceType {
            size: self.size,
            item_type: Box::new(self.item_type.type_of(scope_manager, scope_id)?),
        })))
    }
}
impl CompatibleWith for static_types::SliceType {}
impl TypeOf for StrSliceType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(StaticType::StrSlice(
            static_types::StrSliceType {},
        )))
    }
}

impl TypeOf for StringType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(static_types::StaticType::String(
            static_types::StringType(),
        )))
    }
}
impl CompatibleWith for static_types::StringType {}

impl CompatibleWith for static_types::StrSliceType {}

impl TypeOf for VecType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(static_types::StaticType::Vec(
            static_types::VecType(Box::new(self.0.type_of(scope_manager, scope_id)?)),
        )))
    }
}
impl CompatibleWith for static_types::VecType {}

impl TypeOf for ClosureType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let params = {
            let mut params = Vec::with_capacity(self.params.len());
            for p in &self.params {
                params.push(p.type_of(scope_manager, scope_id)?);
            }
            params
        };
        let scope_params_size =
            params.iter().map(|p| p.size_of()).sum::<usize>() + if self.closed { 8 } else { 0 };
        let static_type: StaticType =
            static_types::StaticType::Closure(static_types::ClosureType {
                params,
                ret: Box::new(self.ret.type_of(scope_manager, scope_id)?),
                closed: self.closed,
                scope_params_size,
            });
        Ok(e_static!(static_type))
    }
}

impl CompatibleWith for static_types::FnType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.params.len() != other.params.len()
            || self.scope_params_size != other.scope_params_size
        {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_param, other_param) in self.params.iter().zip(other.params.iter()) {
            let _ = self_param.compatible_with(&other_param, scope_manager, scope_id)?;
        }
        return self
            .ret
            .compatible_with(&other.ret, scope_manager, scope_id);
    }
}

impl CompatibleWith for static_types::ClosureType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.params.len() != other.params.len()
                    // || self.rec != *other.rec
                    || self.closed != other.closed
        {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_param, other_param) in self.params.iter().zip(other.params.iter()) {
            let _ = self_param.compatible_with(&other_param, scope_manager, scope_id)?;
        }
        return self
            .ret
            .compatible_with(&other.ret, scope_manager, scope_id);
    }
}

impl TypeOf for TupleType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let mut vec = Vec::with_capacity(self.0.len());
        for subtype in &self.0 {
            let subtype = subtype.type_of(&scope_manager, scope_id)?;
            vec.push(subtype);
        }

        Ok(EType::Static(static_types::StaticType::Tuple(
            static_types::TupleType(vec),
        )))
    }
}

impl CompatibleWith for static_types::TupleType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.0.len() != other.0.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_param, other_param) in self.0.iter().zip(other.0.iter()) {
            let _ = self_param.compatible_with(&other_param, scope_manager, scope_id)?;
        }
        return Ok(());
    }
}

impl TypeOf for AddrType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(static_types::StaticType::Address(
            static_types::AddrType(Box::new(self.0.type_of(scope_manager, scope_id)?)),
        )))
    }
}

impl CompatibleWith for static_types::AddrType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        return self.0.compatible_with(&other.0, scope_manager, scope_id);
    }
}

impl TypeOf for RangeType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        Ok(EType::Static(static_types::StaticType::Range(
            static_types::RangeType {
                num: match self.num {
                    super::NumberType::U8 => static_types::NumberType::U8,
                    super::NumberType::U16 => static_types::NumberType::U16,
                    super::NumberType::U32 => static_types::NumberType::U32,
                    super::NumberType::U64 => static_types::NumberType::U64,
                    super::NumberType::U128 => static_types::NumberType::U128,
                    super::NumberType::I8 => static_types::NumberType::I8,
                    super::NumberType::I16 => static_types::NumberType::I16,
                    super::NumberType::I32 => static_types::NumberType::I32,
                    super::NumberType::I64 => static_types::NumberType::I64,
                    super::NumberType::I128 => static_types::NumberType::I128,
                    super::NumberType::F64 => static_types::NumberType::F64,
                },
                inclusive: self.inclusive,
            },
        )))
    }
}

impl TypeOf for MapType {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let key_type = self.keys_type.type_of(&scope_manager, scope_id)?;
        let subtype = self.values_type.type_of(&scope_manager, scope_id)?;
        Ok(EType::Static(static_types::StaticType::Map(
            static_types::MapType {
                keys_type: Box::new(key_type),
                values_type: Box::new(subtype),
            },
        )))
    }
}
