use std::{cell::Ref, cmp::max};

use crate::{
    e_static,
    semantic::{scope::user_type_impl::UserType, EType, Either, MergeType, SemanticError, TypeOf},
};

use super::{
    AddrType, ChanType, ClosureType, FnType, MapType, PrimitiveType, RangeType, SliceType,
    StaticType, StrSliceType, StringType, TupleType, VecType,
};
use crate::semantic::scope::scope::Scope;

impl MergeType for StaticType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, Self>, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let StaticType::Unit = self {
            return Ok(other_type);
        }

        let Either::Static(static_other_type) = &other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match static_other_type.as_ref() {
            Self::Error => return Ok(e_static!(StaticType::Error)),
            Self::Unit => return Ok(e_static!(self.clone())),
            _ => {}
        }
        match self {
            StaticType::Primitive(value) => value.merge(other, scope),
            StaticType::Slice(value) => value.merge(other, scope),
            StaticType::Vec(value) => value.merge(other, scope),
            StaticType::StaticFn(value) => value.merge(other, scope),
            StaticType::Closure(value) => value.merge(other, scope),
            StaticType::Chan(value) => value.merge(other, scope),
            StaticType::Tuple(value) => value.merge(other, scope),
            StaticType::Unit => Ok(other_type),
            StaticType::Any => Err(SemanticError::CantInferType),
            StaticType::Error => Ok(e_static!(StaticType::Error)),
            StaticType::Address(value) => value.merge(other, scope),
            StaticType::Map(value) => value.merge(other, scope),
            StaticType::String(value) => value.merge(other, scope),
            StaticType::StrSlice(value) => value.merge(other, scope),
            StaticType::Range(value) => value.merge(other, scope),
        }
    }
}

impl MergeType for PrimitiveType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
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
            PrimitiveType::Number(left) => match other_type {
                PrimitiveType::Number(right) => (left == right)
                    .then_some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(*left)).into(),
                    ))
                    .ok_or(SemanticError::IncompatibleTypes),
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

impl MergeType for SliceType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Slice(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };

        if self.size != other_type.size {
            Err(SemanticError::IncompatibleTypes)
        } else {
            let merged = self.item_type.merge(other_type.item_type.as_ref(), scope)?;
            Ok(Either::Static(
                StaticType::Slice(SliceType {
                    size: self.size.clone(),
                    item_type: Box::new(merged),
                })
                .into(),
            ))
        }
    }
}

impl MergeType for RangeType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Range(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };

        if self.num != other_type.num || self.inclusive != other_type.inclusive {
            return Err(SemanticError::IncompatibleTypes);
        }

        Ok(Either::Static(
            StaticType::Range(RangeType {
                num: self.num,
                inclusive: self.inclusive,
            })
            .into(),
        ))
    }
}

impl MergeType for StringType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::String(_other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        Ok(e_static!(StaticType::String(StringType())))
    }
}

impl MergeType for StrSliceType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        if let Either::Static(other_type) = other_type {
            if let StaticType::StrSlice(StrSliceType { size: other_size }) = other_type.as_ref() {
                return Ok(Either::Static(
                    StaticType::StrSlice(StrSliceType {
                        size: self.size + other_size,
                    })
                    .into(),
                ));
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        } else {
            return Err(SemanticError::IncompatibleTypes);
        }
    }
}

impl MergeType for VecType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
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

impl MergeType for FnType {
    fn merge<Other>(&self, _other: &Other, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        Err(SemanticError::IncompatibleTypes)
        // let other_type = other.type_of(&block)?;
        // let Either::Static(other_type) = other_type else {
        //     return Err(SemanticError::IncompatibleTypes);
        // };
        // let StaticType::StaticFn(other_type) = other_type.as_ref() else {
        //     return Err(SemanticError::IncompatibleTypes);
        // };
        // if self.params.len() != other_type.params.len() {
        //     return Err(SemanticError::IncompatibleTypes);
        // }
        // let mut merged_params = Vec::with_capacity(self.params.len());
        // for (self_inner, other_inner) in self.params.iter().zip(other_type.params.iter()) {
        //     let merged = self_inner.merge(other_inner, block)?;
        //     merged_params.push(merged);
        // }
        // let merged_ret = self.ret.merge(other_type.ret.as_ref(), block)?;
        // Ok(Either::Static(
        //     StaticType::StaticFn(FnType {
        //         params: merged_params,
        //         ret: Box::new(merged_ret),
        //     })
        //     .into(),
        // ))
    }
}

impl MergeType for ClosureType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Closure(other_type) = other_type.as_ref() else {
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

        if self.closed != other_type.closed {
            return Err(SemanticError::IncompatibleTypes);
        }
        let scope_params_size = max(self.scope_params_size, other_type.scope_params_size);
        Ok(Either::Static(
            StaticType::Closure(ClosureType {
                params: merged_params,
                ret: Box::new(merged_ret),
                closed: self.closed,
                scope_params_size,
            })
            .into(),
        ))
    }
}

impl MergeType for ChanType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
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

impl MergeType for TupleType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
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
            let merged = self_inner.merge(other_inner, scope)?;
            merged_vec.push(merged);
        }
        Ok(Either::Static(
            StaticType::Tuple(TupleType(merged_vec)).into(),
        ))
    }
}

impl MergeType for AddrType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
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

impl MergeType for MapType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        let other_type = other.type_of(&scope)?;
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let StaticType::Map(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };

        let merged_key = self.keys_type.merge(other_type.keys_type.as_ref(), scope)?;

        let merged_value = self
            .values_type
            .merge(other_type.values_type.as_ref(), scope)?;
        Ok(Either::Static(
            StaticType::Map(MapType {
                keys_type: Box::new(merged_key),
                values_type: Box::new(merged_value),
            })
            .into(),
        ))
    }
}
