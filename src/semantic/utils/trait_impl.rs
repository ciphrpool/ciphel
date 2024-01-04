use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{
            type_traits::{GetSubTypes, OperandMerging, TypeChecking},
            ScopeApi,
        },
        CompatibleWith, EitherType, MergeType, Resolve, SemanticError, TypeOf,
    },
};

impl<T, Scope: ScopeApi> CompatibleWith<Scope> for Option<T>
where
    T: CompatibleWith<Scope>,
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            Some(value) => value.compatible_with(other, scope),
            None => {
                let other_type = other.type_of(scope)?;
                if <EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType> as TypeChecking<Scope>>::is_unit(&other_type) {
                    Ok(())
                } else {
                    Err(SemanticError::IncompatibleTypes)
                }
            }
        }
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
        Scope: ScopeApi,
        Self: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(static_type) => static_type.compatible_with(other, scope),
            EitherType::User(user_type) => user_type.compatible_with(other, scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        match self {
            EitherType::Static(static_type) => static_type.type_of(scope),
            EitherType::User(user_type) => user_type.type_of(scope),
        }
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(static_type) => static_type.merge(other, scope),
            EitherType::User(user_type) => user_type.merge(other, scope),
        }
    }
}

impl<Scope: ScopeApi> TypeChecking<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn is_iterable(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_iterable(),
            EitherType::User(_) => false,
        }
    }

    fn is_boolean(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_boolean(),
            EitherType::User(_) => false,
        }
    }
    fn is_enum_variant(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_enum_variant(),
            EitherType::User(user_type) => user_type.is_enum_variant(),
        }
    }
    fn is_callable(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_callable(),
            EitherType::User(user_type) => user_type.is_callable(),
        }
    }

    fn is_channel(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_channel(),
            EitherType::User(user_type) => false,
        }
    }

    fn is_any(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_any(),
            EitherType::User(user_type) => user_type.is_any(),
        }
    }
    fn is_unit(&self) -> bool {
        match self {
            EitherType::Static(static_type) => static_type.is_unit(),
            EitherType::User(user_type) => false,
        }
    }
}

impl<Scope: ScopeApi> GetSubTypes<Scope> for EitherType<Scope::UserType, Scope::StaticType> {
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(static_type) => static_type.get_nth(n),
            EitherType::User(user_type) => user_type.get_nth(n),
        }
    }

    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(_) => None,
            EitherType::User(user_type) => user_type.get_field(field_id),
        }
    }
    fn get_variant(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            EitherType::Static(_) => None,
            EitherType::User(user_type) => user_type.get_field(field_id),
        }
    }
    fn get_item(
        &self,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            EitherType::Static(static_type) => static_type.get_item(),
            EitherType::User(_) => None,
        }
    }
    fn get_return(
        &self,
    ) -> Option<EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>> {
        match self {
            EitherType::Static(static_type) => static_type.get_return(),
            EitherType::User(_) => None,
        }
    }

    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        )>,
    > {
        match self {
            EitherType::Static(static_type) => static_type.get_fields(),
            EitherType::User(user_type) => user_type.get_fields(),
        }
    }
}

impl<User, Static, Scope: ScopeApi> OperandMerging<Scope> for EitherType<User, Static>
where
    User: CompatibleWith<Scope> + TypeOf<Scope> + OperandMerging<Scope>,
    Static: CompatibleWith<Scope> + TypeOf<Scope> + OperandMerging<Scope>,
{
    fn can_minus(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_minus(),
            EitherType::User(value) => value.can_minus(),
        }
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_negate(),
            EitherType::User(value) => value.can_negate(),
        }
    }

    fn can_high_ord_math(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_high_ord_math(),
            EitherType::User(value) => value.can_high_ord_math(),
        }
    }
    fn merge_high_ord_math<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_high_ord_math(other, scope),
            EitherType::User(value) => value.merge_high_ord_math(other, scope),
        }
    }

    fn can_low_ord_math(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_low_ord_math(),
            EitherType::User(value) => value.can_low_ord_math(),
        }
    }
    fn merge_low_ord_math<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_low_ord_math(other, scope),
            EitherType::User(value) => value.merge_low_ord_math(other, scope),
        }
    }

    fn can_shift(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_shift(),
            EitherType::User(value) => value.can_shift(),
        }
    }
    fn merge_shift<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_shift(other, scope),
            EitherType::User(value) => value.merge_shift(other, scope),
        }
    }

    fn can_bitwise_and(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_bitwise_and(),
            EitherType::User(value) => value.can_bitwise_and(),
        }
    }
    fn merge_bitwise_and<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_bitwise_and(other, scope),
            EitherType::User(value) => value.merge_bitwise_and(other, scope),
        }
    }

    fn can_bitwise_xor(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_bitwise_xor(),
            EitherType::User(value) => value.can_bitwise_xor(),
        }
    }
    fn merge_bitwise_xor<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_bitwise_xor(other, scope),
            EitherType::User(value) => value.merge_bitwise_xor(other, scope),
        }
    }

    fn can_bitwise_or(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_bitwise_or(),
            EitherType::User(value) => value.can_bitwise_or(),
        }
    }
    fn merge_bitwise_or<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_bitwise_or(other, scope),
            EitherType::User(value) => value.merge_bitwise_or(other, scope),
        }
    }

    fn can_comparaison(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_comparaison(),
            EitherType::User(value) => value.can_comparaison(),
        }
    }
    fn merge_comparaison<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_comparaison(other, scope),
            EitherType::User(value) => value.merge_comparaison(other, scope),
        }
    }

    fn can_logical_and(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_logical_and(),
            EitherType::User(value) => value.can_logical_and(),
        }
    }
    fn merge_logical_and<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_logical_and(other, scope),
            EitherType::User(value) => value.merge_logical_and(other, scope),
        }
    }

    fn can_logical_or(&self) -> Result<(), SemanticError> {
        match self {
            EitherType::Static(value) => value.can_logical_or(),
            EitherType::User(value) => value.can_logical_or(),
        }
    }
    fn merge_logical_or<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            EitherType::Static(value) => value.merge_logical_or(other, scope),
            EitherType::User(value) => value.merge_logical_or(other, scope),
        }
    }
}
