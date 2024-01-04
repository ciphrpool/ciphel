use crate::{
    ast::utils::strings::ID,
    semantic::{EitherType, SemanticError, TypeOf},
};

use super::ScopeApi;

pub trait GetSubTypes<Scope: ScopeApi> {
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_field(&self, field_id: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_variant(&self, variant: &ID) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_return(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        None
    }
    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
        None
    }
}

pub trait IsEnum {
    fn is_enum(&self) -> bool {
        false
    }
}
pub trait TypeChecking<Scope: ScopeApi> {
    fn is_iterable(&self) -> bool {
        false
    }
    fn is_channel(&self) -> bool {
        false
    }
    fn is_boolean(&self) -> bool {
        false
    }
    fn is_enum_variant(&self) -> bool {
        false
    }

    fn is_callable(&self) -> bool {
        false
    }
    fn is_any(&self) -> bool {
        false
    }
    fn is_unit(&self) -> bool {
        false
    }
}

pub trait OperandMerging<Scope: ScopeApi> {
    fn can_minus(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn can_high_ord_math(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_high_ord_math<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_low_ord_math(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_low_ord_math<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_shift(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_shift<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_bitwise_and(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_bitwise_and<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_bitwise_xor(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_bitwise_xor<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_bitwise_or(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_bitwise_or<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_comparaison(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_comparaison<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_logical_and(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_logical_and<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_logical_or(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_logical_or<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }
}
