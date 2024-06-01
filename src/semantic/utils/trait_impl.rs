use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::{
    ast::{expressions::data::Data, utils::strings::ID},
    semantic::{
        scope::{
            static_types::{AddrType, StaticType},
            type_traits::{GetSubTypes, OperandMerging, TypeChecking},
        },
        CompatibleWith, EType, Either, MergeType, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, DeserializeFrom, NextItem, Printer, RuntimeError},
    },
};

impl<T> CompatibleWith for Option<T>
where
    T: CompatibleWith,
{
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Some(value) => value.compatible_with(other, scope),
            None => {
                Ok(())

                // let other_type = other.type_of(&block)?;

                // if <EType as TypeChecking>::is_unit(&other_type) {
                //     Ok(())
                // } else {
                //     Err(SemanticError::IncompatibleTypes)
                // }
            }
        }
    }
}

impl CompatibleWith for EType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf,

        Self: TypeOf,
    {
        match self {
            Either::Static(static_type) => static_type.compatible_with(other, scope),
            Either::User(user_type) => user_type.compatible_with(other, scope),
        }
    }
}

impl<S: SizeOf, U: SizeOf> SizeOf for Either<S, U> {
    fn size_of(&self) -> usize {
        match self {
            Either::Static(value) => value.size_of(),
            Either::User(value) => value.size_of(),
        }
    }
}

impl TypeOf for EType {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Either::Static(static_type) => static_type.type_of(&scope),
            Either::User(user_type) => user_type.type_of(&scope),
        }
    }
}

impl MergeType for EType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(static_type) => static_type.merge(other, scope),
            Either::User(user_type) => user_type.merge(other, scope),
        }
    }
}

impl TypeChecking for EType {
    fn is_iterable(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_iterable(),
            Either::User(_) => false,
        }
    }

    fn is_boolean(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_boolean(),
            Either::User(_) => false,
        }
    }
    fn is_callable(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_callable(),
            Either::User(user_type) => user_type.is_callable(),
        }
    }

    fn is_any(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_any(),
            Either::User(user_type) => user_type.is_any(),
        }
    }
    fn is_unit(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_unit(),
            Either::User(_user_type) => false,
        }
    }

    fn is_indexable(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_indexable(),
            Either::User(_user_type) => false,
        }
    }

    fn is_dotnum_indexable(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_dotnum_indexable(),
            Either::User(_user_type) => false,
        }
    }

    fn is_addr(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_addr(),
            Either::User(_user_type) => false,
        }
    }

    fn is_u64(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_u64(),
            Either::User(_user_type) => false,
        }
    }

    fn is_char(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_char(),
            Either::User(_user_type) => false,
        }
    }

    fn is_vec(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_vec(),
            Either::User(_user_type) => false,
        }
    }
    fn is_map(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_map(),
            Either::User(_user_type) => false,
        }
    }
    fn is_string(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_string(),
            Either::User(_user_type) => false,
        }
    }
    fn is_err(&self) -> bool {
        match self {
            Either::Static(static_type) => static_type.is_err(),
            Either::User(_user_type) => false,
        }
    }
}

impl GetSubTypes for EType {
    fn get_nth(&self, n: &usize) -> Option<EType> {
        match self {
            Either::Static(static_type) => static_type.get_nth(n),
            Either::User(user_type) => user_type.get_nth(n),
        }
    }

    fn get_field(&self, field_id: &ID) -> Option<EType> {
        match self {
            Either::Static(value) => match value.as_ref() {
                StaticType::Address(AddrType(value)) => value.get_field(field_id),
                _ => None,
            },
            Either::User(user_type) => user_type.get_field(field_id),
        }
    }
    fn get_variant(&self, field_id: &ID) -> Option<EType> {
        match self {
            Either::Static(value) => match value.as_ref() {
                StaticType::Address(AddrType(value)) => value.get_variant(field_id),
                _ => None,
            },
            Either::User(user_type) => user_type.get_variant(field_id),
        }
    }
    fn get_item(&self) -> Option<EType> {
        match self {
            Either::Static(static_type) => static_type.get_item(),
            Either::User(_) => None,
        }
    }

    fn get_key(&self) -> Option<EType> {
        match self {
            Either::Static(static_type) => static_type.get_key(),
            Either::User(_) => None,
        }
    }

    fn get_return(&self) -> Option<EType> {
        match self {
            Either::Static(static_type) => static_type.get_return(),
            Either::User(_) => None,
        }
    }

    fn get_fields(&self) -> Option<Vec<(Option<ID>, EType)>> {
        match self {
            Either::Static(static_type) => static_type.get_fields(),
            Either::User(user_type) => user_type.get_fields(),
        }
    }

    fn get_length(&self) -> Option<usize> {
        match self {
            Either::Static(static_type) => static_type.get_length(),
            Either::User(user_type) => user_type.get_length(),
        }
    }

    fn get_field_offset(&self, field_id: &ID) -> Option<usize> {
        match self {
            Either::Static(value) => match value.as_ref() {
                StaticType::Address(AddrType(value)) => value.get_field_offset(field_id),
                _ => None,
            },
            Either::User(user_type) => user_type.get_field_offset(field_id),
        }
    }

    fn get_inline_field_offset(&self, index: usize) -> Option<usize> {
        match self {
            Either::Static(static_type) => static_type.get_inline_field_offset(index),
            Either::User(_) => None,
        }
    }
}

impl<User, Static> OperandMerging for Either<User, Static>
where
    User: CompatibleWith + TypeOf + OperandMerging,
    Static: CompatibleWith + TypeOf + OperandMerging,
{
    fn can_substract(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_substract(),
            Either::User(value) => value.can_substract(),
        }
    }

    fn merge_substraction<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_substraction(other, scope),
            Either::User(value) => value.merge_substraction(other, scope),
        }
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_negate(),
            Either::User(value) => value.can_negate(),
        }
    }

    fn can_product(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_product(),
            Either::User(value) => value.can_product(),
        }
    }
    fn merge_product<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_product(other, scope),
            Either::User(value) => value.merge_product(other, scope),
        }
    }

    fn can_add(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_add(),
            Either::User(value) => value.can_add(),
        }
    }
    fn merge_addition<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_addition(other, scope),
            Either::User(value) => value.merge_addition(other, scope),
        }
    }

    fn can_shift(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_shift(),
            Either::User(value) => value.can_shift(),
        }
    }
    fn merge_shift<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_shift(other, scope),
            Either::User(value) => value.merge_shift(other, scope),
        }
    }

    fn can_bitwise_and(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_bitwise_and(),
            Either::User(value) => value.can_bitwise_and(),
        }
    }
    fn merge_bitwise_and<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_bitwise_and(other, scope),
            Either::User(value) => value.merge_bitwise_and(other, scope),
        }
    }

    fn can_bitwise_xor(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_bitwise_xor(),
            Either::User(value) => value.can_bitwise_xor(),
        }
    }
    fn merge_bitwise_xor<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_bitwise_xor(other, scope),
            Either::User(value) => value.merge_bitwise_xor(other, scope),
        }
    }

    fn can_bitwise_or(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_bitwise_or(),
            Either::User(value) => value.can_bitwise_or(),
        }
    }
    fn merge_bitwise_or<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_bitwise_or(other, scope),
            Either::User(value) => value.merge_bitwise_or(other, scope),
        }
    }
    fn cast<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.cast(other, scope),
            Either::User(value) => value.cast(other, scope),
        }
    }
    fn can_comparaison(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_comparaison(),
            Either::User(value) => value.can_comparaison(),
        }
    }
    fn merge_comparaison<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_comparaison(other, scope),
            Either::User(value) => value.merge_comparaison(other, scope),
        }
    }

    fn can_equate(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_equate(),
            Either::User(value) => value.can_equate(),
        }
    }
    fn merge_equation<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_equation(other, scope),
            Either::User(value) => value.merge_equation(other, scope),
        }
    }

    fn can_logical_and(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_logical_and(),
            Either::User(value) => value.can_logical_and(),
        }
    }
    fn merge_logical_and<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_logical_and(other, scope),
            Either::User(value) => value.merge_logical_and(other, scope),
        }
    }

    fn can_logical_or(&self) -> Result<(), SemanticError> {
        match self {
            Either::Static(value) => value.can_logical_or(),
            Either::User(value) => value.can_logical_or(),
        }
    }
    fn merge_logical_or<Other>(
        &self,
        other: &Other,
        scope: &Ref<Scope>,
    ) -> Result<EType, SemanticError>
    where
        Other: TypeOf,
    {
        match self {
            Either::Static(value) => value.merge_logical_or(other, scope),
            Either::User(value) => value.merge_logical_or(other, scope),
        }
    }
}

impl DeserializeFrom for EType {
    type Output = Data;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            Either::Static(value) => value.deserialize_from(bytes),
            Either::User(value) => value.deserialize_from(bytes),
        }
    }
}

impl Printer for EType {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            Either::Static(value) => value.build_printer(instructions),
            Either::User(value) => value.build_printer(instructions),
        }
    }
}

impl NextItem for EType {
    fn init_address(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            Either::Static(value) => value.init_address(instructions),
            Either::User(_value) => Err(CodeGenerationError::UnresolvedError),
        }
    }
    fn init_index(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            Either::Static(value) => value.init_index(instructions),
            Either::User(_value) => Err(CodeGenerationError::UnresolvedError),
        }
    }

    fn build_item(
        &self,
        instructions: &CasmProgram,
        end_label: ulid::Ulid,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Either::Static(value) => value.build_item(instructions, end_label),
            Either::User(_value) => Err(CodeGenerationError::UnresolvedError),
        }
    }

    fn next(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            Either::Static(value) => value.next(instructions),
            Either::User(_value) => Err(CodeGenerationError::UnresolvedError),
        }
    }
}
