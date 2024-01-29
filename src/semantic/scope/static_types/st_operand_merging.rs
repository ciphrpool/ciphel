use std::cell::Ref;

use nom::combinator::value;

use crate::semantic::{
    scope::{
        chan_impl::Chan, event_impl::Event, type_traits::OperandMerging, user_type_impl::UserType,
        var_impl::Var, ScopeApi,
    },
    Either, MergeType, SemanticError, TypeOf,
};

use super::{PrimitiveType, SliceType, StaticType, VecType};

impl<Scope: ScopeApi> OperandMerging<Scope> for StaticType {
    fn can_substract(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_substract(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_substraction<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                PrimitiveType::Float => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Number(_)) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    StaticType::Primitive(PrimitiveType::Float) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_addition(
                    &value.0, &other, scope,
                )
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
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_negate(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn can_product(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_product(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_product<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                PrimitiveType::Float => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Number(_)) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    StaticType::Primitive(PrimitiveType::Float) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_product(
                    &value.0, &other, scope,
                )
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_add(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number(_) => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Slice(SliceType::String) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_add(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_addition<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                PrimitiveType::Float => match other_type.as_ref() {
                    StaticType::Primitive(PrimitiveType::Number(_)) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    StaticType::Primitive(PrimitiveType::Float) => Ok(Either::Static(
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Slice(SliceType::String) => match other_type.as_ref() {
                StaticType::Slice(SliceType::String) => {
                    Ok(Either::Static(StaticType::Slice(SliceType::String).into()))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_addition(
                    &value.0, &other, scope,
                )
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
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_shift(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_shift<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_shift(
                    &value.0, &other, scope,
                )
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_and<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_bitwise_and(
                    &value.0, &other, scope,
                )
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_xor<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_bitwise_and(
                    &value.0, &other, scope,
                )
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_bitwise_and(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_bitwise_or<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                        StaticType::Primitive(PrimitiveType::Float).into(),
                    )),
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::merge_bitwise_and(
                    &value.0, &other, scope,
                )
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn cast<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
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
                StaticType::Slice(SliceType::String) => {
                    Ok(Either::Static(Self::Slice(SliceType::String).into()))
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
            StaticType::Slice(SliceType::String) => match other_type.as_ref() {
                Self::Slice(SliceType::String) => {
                    Ok(Either::Static(Self::Slice(SliceType::String).into()))
                }
                StaticType::Slice(SliceType::List(size, subtype)) => {
                    let casted = subtype.as_ref().cast(
                        &Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                        scope,
                    )?;
                    Ok(Either::Static(
                        StaticType::Slice(SliceType::List(*size, Box::new(casted))).into(),
                    ))
                }
                Self::Vec(other_subtype) => {
                    let casted = Either::<UserType, StaticType>::Static(
                        StaticType::Primitive(PrimitiveType::Char).into(),
                    )
                    .cast(other_subtype.0.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Vec(VecType(Box::new(casted))).into(),
                    ))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Slice(SliceType::List(size, subtype)) => match other_type.as_ref() {
                StaticType::Slice(SliceType::List(other_size, other_subtype)) => {
                    if size < &other_size {
                        return Err(SemanticError::IncompatibleOperands);
                    }
                    let casted = subtype.as_ref().cast(other_subtype.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Slice(SliceType::List(*other_size, Box::new(casted))).into(),
                    ))
                }
                StaticType::Slice(SliceType::String) => {
                    let _ = subtype.as_ref().cast(
                        &Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
                        scope,
                    )?;
                    Ok(Either::Static(StaticType::Slice(SliceType::String).into()))
                }
                StaticType::Vec(other_subtype) => {
                    let casted = subtype.cast(other_subtype.0.as_ref(), scope)?;
                    Ok(Either::Static(
                        StaticType::Vec(VecType(Box::new(casted))).into(),
                    ))
                }
                _ => Err(SemanticError::IncompatibleOperands),
            },

            // VEC
            StaticType::Vec(_) => Err(SemanticError::IncompatibleOperands),
            StaticType::Fn(_) => Err(SemanticError::IncompatibleOperands),
            StaticType::Chan(_) => Err(SemanticError::IncompatibleOperands),
            StaticType::Tuple(_) => Err(SemanticError::IncompatibleOperands),
            StaticType::Unit => match other_type.as_ref() {
                StaticType::Unit => Ok(Either::Static(StaticType::Unit.into())),
                _ => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Any => Ok(Either::Static(other_type)),
            StaticType::Error => Ok(Either::Static(StaticType::Error.into())),
            StaticType::Address(_) => Err(SemanticError::IncompatibleOperands),
            StaticType::Map(_) => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_comparaison(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(_) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_comparaison(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_comparaison<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_equate(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_equation<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }

    fn can_include_right(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Slice(_) => Ok(()),
            StaticType::Map(_) => Ok(()),
            StaticType::Vec(_) => Ok(()),
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_include_right(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_inclusion<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_bitwise_or(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_and<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
                <Either<UserType, StaticType> as OperandMerging<Scope>>::can_bitwise_or(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_or<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(Either::Static(
            StaticType::Primitive(PrimitiveType::Bool).into(),
        ))
    }
}
