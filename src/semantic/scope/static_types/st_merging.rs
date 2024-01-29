use std::cell::Ref;

use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, user_type_impl::UserType, var_impl::Var, ScopeApi,
    },
    Either, MergeType, SemanticError, TypeOf,
};

use super::{
    AddrType, ChanType, FnType, KeyType, MapType, NumberType, PrimitiveType, SliceType, StaticType,
    TupleType, VecType,
};

impl<Scope: ScopeApi> MergeType<Scope> for StaticType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, Self>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let StaticType::Unit = self {
            return Ok(other_type);
        }

        let Either::Static(static_other_type) = &other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match static_other_type.as_ref() {
            Self::Error => return Ok(Either::Static(StaticType::Error.into())),
            Self::Unit => return Ok(Either::Static(self.clone().into())),
            _ => {}
        }
        match self {
            StaticType::Primitive(value) => value.merge(other, scope),
            StaticType::Slice(value) => value.merge(other, scope),
            StaticType::Vec(value) => value.merge(other, scope),
            StaticType::Fn(value) => value.merge(other, scope),
            StaticType::Chan(value) => value.merge(other, scope),
            StaticType::Tuple(value) => value.merge(other, scope),
            StaticType::Unit => Ok(other_type),
            StaticType::Any => Err(SemanticError::CantInferType),
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => value.merge(other, scope),
            StaticType::Map(value) => value.merge(other, scope),
        }
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for PrimitiveType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
            PrimitiveType::Number(left) => match other_type {
                PrimitiveType::Number(right) => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(left.merge(right))).into(),
                )),
                PrimitiveType::Float => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Float).into(),
                )),
                _ => Err(SemanticError::IncompatibleTypes),
            },

            PrimitiveType::Float => match other_type {
                PrimitiveType::Number(_) => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Float).into(),
                )),
                PrimitiveType::Float => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Float).into(),
                )),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            PrimitiveType::Char => match other_type {
                PrimitiveType::Char => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Char).into(),
                )),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            PrimitiveType::Bool => match other_type {
                PrimitiveType::Bool => Ok(Either::Static(
                    StaticType::Primitive(PrimitiveType::Bool).into(),
                )),
                _ => Err(SemanticError::IncompatibleTypes),
            },
        }
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for SliceType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Slice(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            SliceType::String => match other_type {
                SliceType::String => {
                    Ok(Either::Static(StaticType::Slice(SliceType::String).into()))
                }
                SliceType::List(_, _) => Err(SemanticError::IncompatibleTypes),
            },
            SliceType::List(size, subtype) => match other_type {
                SliceType::String => Err(SemanticError::IncompatibleTypes),
                SliceType::List(other_size, other_subtype) => {
                    if size != other_size {
                        Err(SemanticError::IncompatibleTypes)
                    } else {
                        let merged = subtype.merge(other_subtype.as_ref(), scope)?;
                        Ok(Either::Static(
                            StaticType::Slice(SliceType::List(size.clone(), Box::new(merged)))
                                .into(),
                        ))
                    }
                }
            },
        }
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for VecType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Vec(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Vec(VecType(Box::new(merged))).into(),
        ))
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for FnType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Fn(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.params.len() != other_type.params.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        let mut merged_params = Vec::with_capacity(self.params.len());
        for (self_inner, other_inner) in self.params.iter().zip(other_type.params.iter()) {
            let merged = self_inner.merge(other_inner, scope)?;
            merged_params.push(merged);
        }
        let merged_ret = self.ret.merge(other_type.ret.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Fn(FnType {
                params: merged_params,
                ret: Box::new(merged_ret),
            })
            .into(),
        ))
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for ChanType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Chan(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Chan(ChanType(Box::new(merged))).into(),
        ))
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for TupleType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Tuple(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.0.len() != other_type.0.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        let mut merged_vec = Vec::with_capacity(self.0.len());
        for (self_inner, other_inner) in self.0.iter().zip(other_type.0.iter()) {
            let merged = self_inner.merge(other_inner, scope)? else {
                return Err(SemanticError::CantInferType);
            };
            merged_vec.push(merged);
        }
        Ok(Either::Static(
            StaticType::Tuple(TupleType(merged_vec)).into(),
        ))
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for AddrType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Address(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Address(AddrType(Box::new(merged))).into(),
        ))
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for MapType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Map(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };

        let merged_key = self.keys_type.merge_key(&other_type.keys_type, scope)?;

        let merged_value = self
            .values_type
            .merge(other_type.values_type.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Map(MapType {
                keys_type: merged_key,
                values_type: Box::new(merged_value),
            })
            .into(),
        ))
    }
}

impl KeyType {
    fn merge_key<Scope: ScopeApi>(
        &self,
        other: &KeyType,
        scope: &Ref<Scope>,
    ) -> Result<KeyType, SemanticError> {
        match (self, other) {
            (KeyType::Primitive(value), KeyType::Primitive(other_value)) => {
                match (value, other_value) {
                    (PrimitiveType::Number(left), PrimitiveType::Number(right)) => {
                        Ok(KeyType::Primitive(PrimitiveType::Number(left.merge(right))))
                    }
                    (PrimitiveType::Number(_), PrimitiveType::Float) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Float, PrimitiveType::Number(_)) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Float, PrimitiveType::Float) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Char, PrimitiveType::Char) => {
                        Ok(KeyType::Primitive(PrimitiveType::Char))
                    }
                    (PrimitiveType::Bool, PrimitiveType::Bool) => {
                        Ok(KeyType::Primitive(PrimitiveType::Bool))
                    }
                    _ => Err(SemanticError::IncompatibleTypes),
                }
            }
            (KeyType::Address(value), KeyType::Address(other_value)) => {
                let merged = value.0.merge(other_value.0.as_ref(), scope)?;
                Ok(KeyType::Address(AddrType(Box::new(merged))))
            }
            (KeyType::Slice(value), KeyType::Slice(other_value)) => match (value, other_value) {
                (SliceType::String, SliceType::String) => Ok(KeyType::Slice(SliceType::String)),
                (SliceType::List(size, subtype), SliceType::List(other_size, other_subtype)) => {
                    if size != other_size {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    let merged = subtype.merge(other_subtype.as_ref(), scope)?;
                    Ok(KeyType::Slice(SliceType::List(
                        size.clone(),
                        Box::new(merged),
                    )))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            (KeyType::Enum(value), KeyType::Enum(other_value)) => {
                let merged = value.merge(other_value)?;
                Ok(KeyType::Enum(merged))
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}
