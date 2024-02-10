use std::cell::Ref;

use super::{
    AddrType, ChanType, FnType, MapType, PrimitiveType, SliceType, StringType, TupleType, Type,
    VecType,
};
use crate::semantic::scope::static_types::{self, StaticType};

use crate::semantic::scope::user_type_impl::UserType;

use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::ScopeApi, Either, SemanticError, TypeOf};
use crate::semantic::{CompatibleWith, EType};

impl<Scope: ScopeApi> TypeOf<Scope> for Type {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        match self {
            Type::Primitive(value) => value.type_of(&scope),
            Type::Slice(value) => value.type_of(&scope),
            Type::UserType(value) => {
                let user_type = scope.find_type(value)?;
                let user_type = user_type.type_of(&scope)?;
                Ok(user_type)
            }
            Type::Vec(value) => value.type_of(&scope),
            Type::Fn(value) => value.type_of(&scope),
            Type::Chan(value) => value.type_of(&scope),
            Type::Tuple(value) => value.type_of(&scope),
            Type::Unit => {
                let static_type: StaticType = <StaticType as BuildStaticType<Scope>>::build_unit();
                let static_type = static_type.type_of(&scope)?;
                Ok(static_type)
            }
            Type::Address(value) => value.type_of(&scope),
            Type::Map(value) => value.type_of(&scope),
            Type::String(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PrimitiveType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_primitive(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::PrimitiveType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Primitive(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            static_types::PrimitiveType::Number(_) => match other_type {
                static_types::PrimitiveType::Number(_) => Ok(()),
                static_types::PrimitiveType::Float => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },

            static_types::PrimitiveType::Float => match other_type {
                static_types::PrimitiveType::Number(_) => Ok(()),
                static_types::PrimitiveType::Float => Ok(()),
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

impl<Scope: ScopeApi> TypeOf<Scope> for SliceType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_slice(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for StringType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_string(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::SliceType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
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

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::StringType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::String(static_types::StringType()) = other_type.as_ref() {
                Ok(())
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VecType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_vec(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::VecType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
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

impl<Scope: ScopeApi> TypeOf<Scope> for FnType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_fn(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::FnType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Fn(static_types::FnType {
                params: other_params,
                ret: other_ret,
            }) = other_type.as_ref()
            {
                if self.params.len() != other_params.len() {
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

impl<Scope: ScopeApi> TypeOf<Scope> for ChanType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_chan(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::ChanType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Chan(static_types::ChanType(other_subtype)) = other_type.as_ref() {
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

impl<Scope: ScopeApi> TypeOf<Scope> for TupleType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_tuple(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::TupleType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
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

impl<Scope: ScopeApi> TypeOf<Scope> for AddrType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_addr(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::AddrType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
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

impl<Scope: ScopeApi> TypeOf<Scope> for MapType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: StaticType = StaticType::build_map(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::MapType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::Map(static_types::MapType {
                keys_type: other_key,
                values_type: other_value,
            }) = other_type.as_ref()
            {
                let other_key = match other_key {
                    static_types::KeyType::Primitive(value) => {
                        Either::Static(StaticType::Primitive(value.clone()).into())
                    }
                    static_types::KeyType::Address(value) => {
                        Either::Static(StaticType::Address(value.clone()).into())
                    }
                    static_types::KeyType::String(value) => {
                        Either::Static(StaticType::String(static_types::StringType()).into())
                    }
                    static_types::KeyType::Enum(value) => {
                        Either::User(UserType::Enum(value.clone()).into())
                    }
                };

                let _ = self.keys_type.compatible_with(&other_key, scope)?;

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
impl<Scope: ScopeApi> CompatibleWith<Scope> for static_types::KeyType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            static_types::KeyType::Primitive(value) => value.compatible_with(other, scope),
            static_types::KeyType::Address(value) => value.compatible_with(other, scope),
            static_types::KeyType::String(value) => value.compatible_with(other, scope),
            static_types::KeyType::Enum(value) => value.compatible_with(other, scope),
        }
    }
}
