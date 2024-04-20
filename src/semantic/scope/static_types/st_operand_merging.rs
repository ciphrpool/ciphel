use std::cell::Ref;

use crate::semantic::{
    scope::{type_traits::OperandMerging, user_type_impl::UserType, ScopeApi},
    CompatibleWith, EType, Either, MergeType, SemanticError, TypeOf,
};

use super::{PrimitiveType, SliceType, StaticType, StrSliceType, StringType, VecType};

impl<Scope: ScopeApi> OperandMerging<Scope> for StaticType {
    fn can_substract(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_substract(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_substraction<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_addition(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_negate(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn can_product(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_product(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_product<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_product(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_add(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::StrSlice(_) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_add(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_addition<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::StrSlice(StrSliceType { size: self_size }) => match other_type.as_ref() {
                StaticType::StrSlice(StrSliceType { size: other_size }) => Ok(Either::Static(
                    StaticType::StrSlice(StrSliceType {
                        size: self_size + other_size,
                    })
                    .into(),
                )),
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_addition(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_shift(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_shift(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_shift<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_shift(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_and(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_and<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Bool) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_bitwise_and(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_xor(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_xor<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Bool) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_bitwise_and(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_or(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_or<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other = other.type_of(&scope)?;
        let Either::Static(other_type) = &other else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            StaticType::Primitive(value) => match value {
                num @ PrimitiveType::Number(_) => num
                    .merge(&other, scope)
                    .map_err(|_| SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Bool) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Bool).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::merge_bitwise_and(&value.0, &other, scope)
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn cast<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        if let StaticType::Any = self {
            return Ok(other_type);
        }
        let Either::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type.as_ref() {
            return Ok(Either::Static(StaticType::Error.into()));
        }
        match self {
            // PRIMITIVE
            StaticType::Primitive(PrimitiveType::Char) => match other_type.as_ref() {
                StaticType::Primitive(PrimitiveType::Bool) => {
                    Err(SemanticError::IncompatibleOperands)
                }
                StaticType::Primitive(to) => Ok(Either::Static(Self::Primitive(to.clone()).into())),
                StaticType::String(_) => {
                    Ok(Either::Static(StaticType::String(StringType()).into()))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Primitive(_) => match other_type.as_ref() {
                StaticType::Primitive(res) => {
                    Ok(Either::Static(StaticType::Primitive(res.clone()).into()))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },

            // SLICE
            StaticType::StrSlice(StrSliceType { size: self_size }) => match other_type.as_ref() {
                StaticType::Slice(SliceType { size, item_type }) => {
                    let _ = Either::<UserType, StaticType>::Static(
                        StaticType::Primitive(PrimitiveType::Char).into(),
                    )
                    .compatible_with(item_type.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Slice(SliceType {
                            size: *size,
                            item_type: Box::new(Either::<UserType, StaticType>::Static(
                                StaticType::Primitive(PrimitiveType::Char).into(),
                            )),
                        })
                        .into(),
                    ))
                }
                Self::String(_) => Ok(Either::Static(Self::String(StringType()).into())),
                Self::Vec(other_subtype) => {
                    let _ = Either::<UserType, StaticType>::Static(
                        StaticType::Primitive(PrimitiveType::Char).into(),
                    )
                    .compatible_with(other_subtype.0.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Vec(VecType(Box::new(Either::<UserType, StaticType>::Static(
                            StaticType::Primitive(PrimitiveType::Char).into(),
                        ))))
                        .into(),
                    ))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },

            StaticType::String(_) => match other_type.as_ref() {
                Self::String(_) => Ok(Either::Static(Self::String(StringType()).into())),
                Self::Vec(other_subtype) => {
                    let _ = Either::<UserType, StaticType>::Static(
                        StaticType::Primitive(PrimitiveType::Char).into(),
                    )
                    .compatible_with(other_subtype.0.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Vec(VecType(Box::new(Either::<UserType, StaticType>::Static(
                            StaticType::Primitive(PrimitiveType::Char).into(),
                        ))))
                        .into(),
                    ))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Slice(SliceType { size, item_type }) => match other_type.as_ref() {
                StaticType::Slice(SliceType {
                    size: other_size,
                    item_type: other_subtype,
                }) => {
                    if size < &other_size {
                        return Err(SemanticError::IncompatibleOperands);
                    }
                    let casted = item_type.as_ref().cast(other_subtype.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Slice(SliceType {
                            size: *other_size,
                            item_type: Box::new(casted),
                        })
                        .into(),
                    ))
                }
                StaticType::String(_) => {
                    let _ = item_type.as_ref().cast(
                        &Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                        scope,
                    )?;
                    Ok(Either::Static(StaticType::String(StringType()).into()))
                }
                StaticType::StrSlice(StrSliceType { size: other_size }) => {
                    let _ = Either::<UserType, StaticType>::Static(
                        StaticType::Primitive(PrimitiveType::Char).into(),
                    )
                    .compatible_with(item_type.as_ref(), scope)?;
                    if size != other_size {
                        return Err(SemanticError::IncompatibleOperands);
                    }

                    Ok(Either::Static(
                        StaticType::StrSlice(StrSliceType { size: *other_size }).into(),
                    ))
                }
                StaticType::Vec(other_subtype) => {
                    let casted = item_type.cast(other_subtype.0.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Vec(VecType(Box::new(casted))).into(),
                    ))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },

            // VEC
            StaticType::Unit => match other_type.as_ref() {
                StaticType::Unit => Ok(Either::Static(StaticType::Unit.into())),
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_comparaison(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(_) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_comparaison(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_comparaison<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }

    fn can_equate(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(_) => Ok(()),
            StaticType::Slice(_) => Ok(()),
            StaticType::Tuple(_) => Ok(()),
            StaticType::Map(_) => Ok(()),
            StaticType::Vec(_) => Ok(()),
            StaticType::String(_) => Ok(()),
            StaticType::StrSlice(_) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EType as OperandMerging<Scope>>::can_equate(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_equation<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }

    fn can_logical_and(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_bitwise_or(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_and<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }

    fn can_logical_or(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EType as OperandMerging<Scope>>::can_bitwise_or(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_or<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }
}
