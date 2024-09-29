use super::{
    Address, Call, Closure, ClosureParam, Data, Enum, Format, Lambda, Map, Number, Primitive,
    Printf, PtrAccess, Slice, StrSlice, Struct, Tuple, Union, Variable, Vector,
};
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{self, StaticType, StringType};

use crate::semantic::SizeOf;
use crate::semantic::{EType, Resolve, SemanticError, TypeOf};

impl TypeOf for Data {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Data::Primitive(value) => value.type_of(&scope_manager, scope_id),
            Data::Slice(value) => value.type_of(&scope_manager, scope_id),
            Data::Vec(value) => value.type_of(&scope_manager, scope_id),
            Data::Closure(value) => value.type_of(&scope_manager, scope_id),
            Data::Lambda(value) => value.type_of(&scope_manager, scope_id),
            Data::Tuple(value) => value.type_of(&scope_manager, scope_id),
            Data::Address(value) => value.type_of(&scope_manager, scope_id),
            Data::PtrAccess(value) => value.type_of(&scope_manager, scope_id),
            Data::Variable(value) => value.type_of(&scope_manager, scope_id),
            Data::Unit => Ok(EType::Static(StaticType::Unit)),
            Data::Map(value) => value.type_of(&scope_manager, scope_id),
            Data::Struct(value) => value.type_of(&scope_manager, scope_id),
            Data::Union(value) => value.type_of(&scope_manager, scope_id),
            Data::Enum(value) => value.type_of(&scope_manager, scope_id),
            Data::StrSlice(value) => value.type_of(&scope_manager, scope_id),
            Data::Call(value) => value.type_of(&scope_manager, scope_id),
            Data::Printf(value) => value.type_of(&scope_manager, scope_id),
            Data::Format(value) => value.type_of(&scope_manager, scope_id),
        }
    }
}

impl TypeOf for Variable {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Primitive {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Primitive::Number(num) => match num {
                Number::U8(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::U8),
                ))),
                Number::U16(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::U16),
                ))),
                Number::U32(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::U32),
                ))),
                Number::U64(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::U64),
                ))),
                Number::U128(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::U128),
                ))),
                Number::I8(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::I8),
                ))),
                Number::I16(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::I16),
                ))),
                Number::I32(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::I32),
                ))),
                Number::I64(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::I64),
                ))),
                Number::I128(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::I128),
                ))),
                Number::F64(_) => Ok(EType::Static(StaticType::Primitive(
                    static_types::PrimitiveType::Number(static_types::NumberType::F64),
                ))),
                Number::Unresolved(_) => Err(SemanticError::CantInferType(
                    "of the given number".to_string(),
                )),
            },
            Primitive::Bool(_) => Ok(EType::Static(StaticType::Primitive(
                static_types::PrimitiveType::Bool,
            ))),
            Primitive::Char(_) => Ok(EType::Static(StaticType::Primitive(
                static_types::PrimitiveType::Char,
            ))),
        }
    }
}

impl TypeOf for Slice {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for StrSlice {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::StrSlice(
            static_types::StrSliceType {},
        )))
    }
}

impl TypeOf for Vector {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Tuple {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Closure {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let expr_type = expr.type_of(&scope_manager, scope_id)?;
            params_types.push(expr_type);
        }
        let ret_type = self.block.type_of(&scope_manager, scope_id)?;

        Ok(EType::Static(StaticType::Closure(
            static_types::ClosureType {
                params: params_types,
                ret: Box::new(ret_type),
            },
        )))
    }
}

impl TypeOf for Lambda {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let expr_type = expr.type_of(&scope_manager, scope_id)?;
            params_types.push(expr_type);
        }
        let ret_type = self.block.type_of(&scope_manager, scope_id)?;

        Ok(EType::Static(StaticType::Lambda(
            static_types::LambdaType {
                params: params_types,
                ret: Box::new(ret_type),
            },
        )))
    }
}

impl TypeOf for ClosureParam {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ClosureParam::Full(var) => var.type_of(&scope_manager, scope_id),
            ClosureParam::Minimal(_) => Ok(EType::Static(StaticType::Any)),
        }
    }
}
impl TypeOf for Address {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let addr_type = self.value.type_of(&scope_manager, scope_id)?;

        Ok(EType::Static(StaticType::Address(static_types::AddrType(
            Box::new(addr_type),
        ))))
    }
}
impl TypeOf for PtrAccess {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
impl TypeOf for Struct {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope_manager.find_type_by_name(&self.id, scope_id)?;
        Ok(EType::User {
            id: user_type.id,
            size: user_type.def.size_of(),
        })
    }
}
impl TypeOf for Union {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope_manager.find_type_by_name(&self.typename, scope_id)?;
        Ok(EType::User {
            id: user_type.id,
            size: user_type.def.size_of(),
        })
    }
}

impl TypeOf for Enum {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope_manager.find_type_by_name(&self.typename, scope_id)?;
        Ok(EType::User {
            id: user_type.id,
            size: user_type.def.size_of(),
        })
    }
}
impl TypeOf for Map {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
impl TypeOf for Call {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Printf {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::Unit))
    }
}

impl TypeOf for Format {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(EType::Static(StaticType::String(StringType())))
    }
}
