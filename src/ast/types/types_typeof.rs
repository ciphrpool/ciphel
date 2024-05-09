use std::cell::Ref;

use super::{
    AddrType, ClosureType, MapType, PrimitiveType, RangeType, SliceType, StrSliceType, StringType,
    TupleType, Type, VecType,
};
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::{self, StaticType};
use crate::{e_static, e_user};

use crate::semantic::scope::user_type_impl::UserType;
use crate::semantic::SizeOf;

use crate::semantic::scope::BuildStaticType;
use crate::semantic::{CompatibleWith, EType};
use crate::semantic::{Either, SemanticError, TypeOf};

impl TypeOf for Type {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Type::Primitive(value) => value.type_of(&scope),
            Type::Slice(value) => value.type_of(&scope),
            Type::StrSlice(value) => value.type_of(&scope),
            Type::UserType(value) => {
                let user_type = scope.find_type(value)?;
                let user_type = user_type.type_of(&scope)?;
                Ok(user_type)
            }
            Type::Vec(value) => value.type_of(&scope),
            Type::Closure(value) => value.type_of(&scope),
            Type::Tuple(value) => value.type_of(&scope),
            Type::Unit => {
                let static_type: StaticType = <StaticType as BuildStaticType>::build_unit();
                let static_type = e_static!(static_type);
                Ok(static_type)
            }
            Type::Any => {
                let static_type = e_static!(StaticType::Any);
                Ok(static_type)
            }
            Type::Address(value) => value.type_of(&scope),
            Type::Map(value) => value.type_of(&scope),
            Type::String(value) => value.type_of(&scope),
            Type::Range(value) => value.type_of(&scope),
        }
    }
}
impl TypeOf for PrimitiveType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_primitive(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::PrimitiveType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Primitive(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            static_types::PrimitiveType::Number(self_n) => match other_type {
                static_types::PrimitiveType::Number(other_n) => (self_n == other_n)
                    .then_some(())
                    .ok_or(SemanticError::IncompatibleTypes),
                _ => Err(SemanticError::IncompatibleTypes),
            },

            static_types::PrimitiveType::Char => match other_type {
                static_types::PrimitiveType::Char => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            static_types::PrimitiveType::Bool => match other_type {
                static_types::PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },
        }
    }
}

impl TypeOf for SliceType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_slice(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}
impl TypeOf for StrSliceType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_str_slice(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl TypeOf for StringType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_string(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::SliceType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Slice(static_types::SliceType {
                size: other_size,
                item_type: other_subtype,
            }) = other_type.as_ref()
            {
                if other_size != &self.size {
                    return Err(SemanticError::IncompatibleTypes);
                }
                let subtype = self.item_type.type_of(&scope)?;
                let other_subtype = other_subtype.type_of(&scope)?;
                return subtype.compatible_with(&other_subtype, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl CompatibleWith for static_types::StringType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            match other_type.as_ref() {
                StaticType::String(_) => Ok(()),
                // StaticType::StrSlice(_) => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl CompatibleWith for static_types::StrSliceType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::StrSlice(static_types::StrSliceType { size: other_size }) =
                other_type.as_ref()
            {
                return (other_size >= &self.size)
                    .then_some(())
                    .ok_or(SemanticError::IncompatibleTypes);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl TypeOf for VecType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_vec(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::VecType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Vec(static_types::VecType(other_subtype)) = other_type.as_ref() {
                let subtype = self.0.type_of(&scope)?;
                let other_subtype = other_subtype.type_of(&scope)?;
                return subtype.compatible_with(&other_subtype, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl TypeOf for ClosureType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let params = {
            let mut params = Vec::with_capacity(self.params.len());
            for p in &self.params {
                params.push(p.type_of(scope)?);
            }
            params
        };
        let static_type: StaticType = StaticType::build_closure(
            &params,
            &self.ret.type_of(scope)?,
            self.closed,
            params.iter().map(|p| p.size_of()).sum::<usize>() + if self.closed { 8 } else { 0 },
            scope,
        )?;
        Ok(e_static!(static_type))
    }
}

impl CompatibleWith for static_types::FnType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::StaticFn(static_types::FnType {
                params: other_params,
                ret: other_ret,
                scope_params_size: other_scope_params_size,
            }) = other_type.as_ref()
            {
                if self.params.len() != other_params.len()
                    || self.scope_params_size != *other_scope_params_size
                {
                    return Err(SemanticError::IncompatibleTypes);
                }
                for (self_param, other_param) in self.params.iter().zip(other_params.iter()) {
                    let self_param = self_param.type_of(&scope)?;
                    let other_param = other_param.type_of(&scope)?;
                    let _ = self_param.compatible_with(&other_param, scope)?;
                }
                let ret = self.ret.type_of(&scope)?;
                let other_ret = other_ret.type_of(&scope)?;
                return ret.compatible_with(&other_ret, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl CompatibleWith for static_types::ClosureType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Closure(static_types::ClosureType {
                params: other_params,
                ret: other_ret,
                closed: other_closed,
                scope_params_size: _other_scope_params_size,
            }) = other_type.as_ref()
            {
                if self.params.len() != other_params.len()
                    // || self.rec != *other_rec
                    || self.closed != *other_closed
                {
                    return Err(SemanticError::IncompatibleTypes);
                }
                for (self_param, other_param) in self.params.iter().zip(other_params.iter()) {
                    let self_param = self_param.type_of(&scope)?;
                    let other_param = other_param.type_of(&scope)?;
                    let _ = self_param.compatible_with(&other_param, scope)?;
                }
                let ret = self.ret.type_of(&scope)?;
                let other_ret = other_ret.type_of(&scope)?;
                return ret.compatible_with(&other_ret, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}


impl TypeOf for TupleType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_tuple(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::TupleType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Tuple(static_types::TupleType(other_type)) = other_type.as_ref() {
                if self.0.len() != other_type.len() {
                    return Err(SemanticError::IncompatibleTypes);
                }
                for (self_param, other_param) in self.0.iter().zip(other_type.iter()) {
                    let self_param = self_param.type_of(&scope)?;
                    let other_param = other_param.type_of(&scope)?;
                    let _ = self_param.compatible_with(&other_param, scope)?;
                }
                return Ok(());
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl TypeOf for AddrType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_addr(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::AddrType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Address(static_types::AddrType(other_subtype)) = other_type.as_ref()
            {
                let subtype = self.0.type_of(&scope)?;
                let other_subtype = other_subtype.type_of(&scope)?;
                return subtype.compatible_with(&other_subtype, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl TypeOf for RangeType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_range(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::RangeType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Range(static_types::RangeType {
                num: other_subtype,
                inclusive: other_inclusive,
            }) = other_type.as_ref()
            {
                return (self.num == *other_subtype && self.inclusive == *other_inclusive)
                    .then(|| ())
                    .ok_or(SemanticError::IncompatibleTypes);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl TypeOf for MapType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_map(&self, scope)?;
        let static_type = e_static!(static_type);
        Ok(static_type)
    }
}

impl CompatibleWith for static_types::MapType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Map(static_types::MapType {
                keys_type: other_key,
                values_type: other_value,
            }) = other_type.as_ref()
            {
                let _ = self.keys_type.compatible_with(other_key.as_ref(), scope)?;

                let subtype = self.values_type.type_of(&scope)?;
                let other_subtype = other_value.type_of(&scope)?;
                return subtype.compatible_with(&other_subtype, scope);
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}
