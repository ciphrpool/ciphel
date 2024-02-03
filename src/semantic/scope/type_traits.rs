use std::cell::Ref;

use crate::{
    ast::utils::strings::ID,
    semantic::{Either, SemanticError, TypeOf},
};

use super::{static_types::StaticType, user_type_impl::UserType, ScopeApi};

pub trait GetSubTypes {
    fn get_nth(&self, _n: &usize) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_field(&self, _field_id: &ID) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_variant(&self, _variant: &ID) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_item(&self) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_key(&self) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_length(&self) -> Option<usize> {
        None
    }
    fn get_return(&self) -> Option<Either<UserType, StaticType>> {
        None
    }
    fn get_fields(&self) -> Option<Vec<(Option<String>, Either<UserType, StaticType>)>> {
        None
    }

    fn get_field_offset(&self, _field_id: &ID) -> Option<usize> {
        None
    }

    fn get_inline_field_offset(&self, _index: usize) -> Option<usize> {
        None
    }
}
pub trait IsEnum {
    fn is_enum(&self) -> bool {
        false
    }
}
pub trait TypeChecking {
    fn is_iterable(&self) -> bool {
        false
    }
    fn is_indexable(&self) -> bool {
        false
    }
    fn is_dotnum_indexable(&self) -> bool {
        false
    }
    fn is_channel(&self) -> bool {
        false
    }
    fn is_boolean(&self) -> bool {
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
    fn is_addr(&self) -> bool {
        false
    }
    fn is_u64(&self) -> bool {
        false
    }
    fn is_char(&self) -> bool {
        false
    }
    fn is_vec(&self) -> bool {
        false
    }
    fn is_map(&self) -> bool {
        false
    }
}

pub trait OperandMerging<Scope: ScopeApi> {
    fn can_substract(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_substraction<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn can_product(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_product<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_add(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_addition<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn cast<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_equate(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }
    fn merge_equation<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }

    fn can_include_left(&self) -> Result<(), SemanticError> {
        Ok(())
    }
    fn can_include_right(&self) -> Result<(), SemanticError> {
        Err(SemanticError::IncompatibleOperation)
    }

    fn merge_inclusion<Other>(
        &self,
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
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
        _other: &Other,
        _scope: &Ref<Scope>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Err(SemanticError::IncompatibleOperands)
    }
}
