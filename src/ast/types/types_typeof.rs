use std::cell::Ref;

use super::{
    AddrType, ChanType, FnType, MapType, PrimitiveType, SliceType, TupleType, Type, VecType,
};
use crate::semantic::scope::static_type_impl::{self, StaticType};

use crate::semantic::scope::user_type_impl::UserType;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::CompatibleWith;
use crate::semantic::{scope::ScopeApi, EitherType, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Type {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
                let static_type: Scope::StaticType = Scope::StaticType::build_unit();
                let static_type = static_type.type_of(&scope)?;
                Ok(static_type)
            }
            Type::Address(value) => value.type_of(&scope),
            Type::Map(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PrimitiveType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_primitive(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::PrimitiveType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Primitive(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            static_type_impl::PrimitiveType::Number => match other_type {
                static_type_impl::PrimitiveType::Number => Ok(()),
                static_type_impl::PrimitiveType::Float => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },

            static_type_impl::PrimitiveType::Float => match other_type {
                static_type_impl::PrimitiveType::Number => Ok(()),
                static_type_impl::PrimitiveType::Float => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },

            static_type_impl::PrimitiveType::Char => match other_type {
                static_type_impl::PrimitiveType::Char => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            static_type_impl::PrimitiveType::Bool => match other_type {
                static_type_impl::PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleTypes),
            },
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for SliceType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_slice(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::SliceType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            static_type_impl::SliceType::String => {
                let other_type = other.type_of(&scope)?;
                if let EitherType::Static(StaticType::Slice(static_type_impl::SliceType::String)) =
                    other_type
                {
                    return Ok(());
                } else {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
            static_type_impl::SliceType::List(size, subtype) => {
                let other_type = other.type_of(&scope)?;
                if let EitherType::Static(StaticType::Slice(static_type_impl::SliceType::List(
                    other_size,
                    other_subtype,
                ))) = other_type
                {
                    if &other_size != size {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    let subtype = subtype.type_of(&scope)?;
                    let other_subtype = other_subtype.type_of(&scope)?;
                    return subtype.compatible_with(&other_subtype, scope);
                } else {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VecType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_vec(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::VecType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Vec(static_type_impl::VecType(other_subtype))) =
            other_type
        {
            let subtype = self.0.type_of(&scope)?;
            let other_subtype = other_subtype.type_of(&scope)?;
            return subtype.compatible_with(&other_subtype, scope);
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FnType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_fn(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::FnType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Fn(static_type_impl::FnType {
            params: other_params,
            ret: other_ret,
        })) = other_type
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
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ChanType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_chan(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::ChanType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Chan(static_type_impl::ChanType(other_subtype))) =
            other_type
        {
            let subtype = self.0.type_of(&scope)?;
            let other_subtype = other_subtype.type_of(&scope)?;
            return subtype.compatible_with(&other_subtype, scope);
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for TupleType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_tuple(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::TupleType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Tuple(static_type_impl::TupleType(other_type))) =
            other_type
        {
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
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for AddrType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_addr(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::AddrType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Address(static_type_impl::AddrType(other_subtype))) =
            other_type
        {
            let subtype = self.0.type_of(&scope)?;
            let other_subtype = other_subtype.type_of(&scope)?;
            return subtype.compatible_with(&other_subtype, scope);
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for MapType {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_map(&self, scope)?;
        let static_type = static_type.type_of(&scope)?;
        Ok(static_type)
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::MapType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let EitherType::Static(StaticType::Map(static_type_impl::MapType {
            keys_type: other_key,
            values_type: other_value,
        })) = other_type
        {
            let other_key = match other_key {
                static_type_impl::KeyType::Primitive(value) => {
                    EitherType::Static(StaticType::Primitive(value))
                }
                static_type_impl::KeyType::Address(value) => {
                    EitherType::Static(StaticType::Address(value))
                }
                static_type_impl::KeyType::Slice(value) => {
                    EitherType::Static(StaticType::Slice(value))
                }
                static_type_impl::KeyType::Enum(value) => EitherType::User(UserType::Enum(value)),
            };

            let _ = self.keys_type.compatible_with(&other_key, scope)?;

            let subtype = self.values_type.type_of(&scope)?;
            let other_subtype = other_value.type_of(&scope)?;
            return subtype.compatible_with(&other_subtype, scope);
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}
impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> CompatibleWith<Scope>
    for static_type_impl::KeyType
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            static_type_impl::KeyType::Primitive(value) => value.compatible_with(other, scope),
            static_type_impl::KeyType::Address(value) => value.compatible_with(other, scope),
            static_type_impl::KeyType::Slice(value) => value.compatible_with(other, scope),
            static_type_impl::KeyType::Enum(value) => value.compatible_with(other, scope),
        }
    }
}
