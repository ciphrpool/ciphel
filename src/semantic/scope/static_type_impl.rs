use crate::{
    ast::types::{
        AddrType, ChanType, FnType, MapType, PrimitiveType, SliceType, TupleType, VecType,
    },
    semantic::{CompatibleWith, EitherType, MergeType, Resolve, SemanticError, TypeOf},
};

use super::{
    type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking},
    BuildStaticType, BuildUserType, ScopeApi,
};

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Primitive(PrimitiveType),
    Slice(SliceType),
    Vec(VecType),
    Fn(FnType),
    Chan(ChanType),
    Tuple(TupleType),
    Unit,
    Address(AddrType),
    Map(MapType),
}

impl<Scope: ScopeApi<StaticType = Self>> TypeOf<Scope> for StaticType {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<<Scope as ScopeApi>::UserType, Self>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        Ok(Some(EitherType::Static(self.clone())))
    }
}

impl<Scope: ScopeApi<StaticType = Self>> CompatibleWith<Scope> for StaticType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let Some(other_type) = other.type_of(scope)? else {
            return Err(SemanticError::CantInferType);
        };
        match other_type {
            EitherType::Static(other_type) => match self {
                StaticType::Primitive(_) => {
                    if self == &other_type {
                        Ok(())
                    } else {
                        Err(SemanticError::IncompatibleTypes)
                    }
                }
                StaticType::Slice(_) => todo!(),
                StaticType::Vec(_) => todo!(),
                StaticType::Fn(_) => todo!(),
                StaticType::Chan(_) => todo!(),
                StaticType::Tuple(_) => todo!(),
                StaticType::Unit => todo!(),
                StaticType::Address(_) => todo!(),
                StaticType::Map(_) => todo!(),
            },
            EitherType::User(_) => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl<Scope: ScopeApi<StaticType = Self>> BuildStaticType<Scope> for StaticType {
    fn build_primitive(type_sig: &crate::ast::types::PrimitiveType) -> Self {
        Self::Primitive(type_sig.clone())
    }

    fn build_slice(type_sig: &crate::ast::types::SliceType) -> Self {
        todo!()
    }

    fn build_slice_from(type_sig: &Vec<EitherType<<Scope as ScopeApi>::UserType, Self>>) -> Self {
        todo!()
    }

    fn build_tuple(type_sig: &crate::ast::types::TupleType) -> Self {
        todo!()
    }

    fn build_tuple_from(type_sig: &Vec<EitherType<<Scope as ScopeApi>::UserType, Self>>) -> Self {
        todo!()
    }

    fn build_vec(type_sig: &crate::ast::types::VecType) -> Self {
        todo!()
    }

    fn build_vec_from(type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>) -> Self {
        todo!()
    }

    fn build_error(type_sig: &crate::ast::expressions::error::Error) -> Self {
        todo!()
    }

    fn build_fn(type_sig: &crate::ast::types::FnType) -> Self {
        todo!()
    }

    fn build_fn_from(
        params: &Vec<EitherType<<Scope as ScopeApi>::UserType, Self>>,
        ret: &EitherType<<Scope as ScopeApi>::UserType, Self>,
    ) -> Self {
        todo!()
    }

    fn build_chan(type_sig: &crate::ast::types::ChanType) -> Self {
        todo!()
    }

    fn build_chan_from(type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>) -> Self {
        todo!()
    }

    fn build_unit() -> Self {
        todo!()
    }

    fn build_any() -> Self {
        todo!()
    }

    fn build_addr(type_sig: &crate::ast::types::AddrType) -> Self {
        todo!()
    }

    fn build_addr_from(type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>) -> Self {
        todo!()
    }

    fn build_ptr_access_from(type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>) -> Self {
        todo!()
    }

    fn build_map(type_sig: &crate::ast::types::MapType) -> Self {
        todo!()
    }

    fn build_map_from(
        key: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        value: &EitherType<<Scope as ScopeApi>::UserType, Self>,
    ) -> Self {
        todo!()
    }
}

impl<Scope: ScopeApi<StaticType = Self>> GetSubTypes<Scope> for StaticType {}

impl<Scope: ScopeApi<StaticType = Self>> TypeChecking<Scope> for StaticType {}

impl<Scope: ScopeApi<StaticType = Self>> OperandMerging<Scope> for StaticType {}

impl<Scope: ScopeApi<StaticType = Self>> MergeType<Scope> for StaticType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<Option<EitherType<<Scope as ScopeApi>::UserType, Self>>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        todo!()
    }
}
