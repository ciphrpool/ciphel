
use super::{
    Address, Closure, ClosureParam, Data, Enum, ExprScope, Map, Number, Primitive, PtrAccess,
    Slice, StrSlice, Struct, Tuple, Union, Variable, Vector,
};
use crate::ast::types::{NumberType, PrimitiveType, StrSliceType};

use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::StaticType;
use crate::{arw_read, e_static};

use crate::semantic::scope::BuildStaticType;
use crate::semantic::{Either, Resolve, SemanticError, TypeOf};
use crate::semantic::{EType, SizeOf};

impl TypeOf for Data {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Data::Primitive(value) => value.type_of(&scope),
            Data::Slice(value) => value.type_of(&scope),
            Data::Vec(value) => value.type_of(&scope),
            Data::Closure(value) => value.type_of(&scope),
            Data::Tuple(value) => value.type_of(&scope),
            Data::Address(value) => value.type_of(&scope),
            Data::PtrAccess(value) => value.type_of(&scope),
            Data::Variable(value) => value.type_of(&scope),
            Data::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_unit().into(),
            )),
            Data::Map(value) => value.type_of(&scope),
            Data::Struct(value) => value.type_of(&scope),
            Data::Union(value) => value.type_of(&scope),
            Data::Enum(value) => value.type_of(&scope),
            Data::StrSlice(value) => value.type_of(&scope),
        }
    }
}

impl TypeOf for Variable {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Primitive {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Primitive::Number(num) => match num {
                Number::U8(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U8), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::U16(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U16), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::U32(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U32), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::U64(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U64), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::U128(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U128), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::I8(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I8), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::I16(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I16), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::I32(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I32), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::I64(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I64), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::I128(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I128), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::F64(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::F64), scope)
                        .map(|value| (e_static!(value)))
                }
                Number::Unresolved(_) => Err(SemanticError::CantInferType),
            },
            Primitive::Bool(_) => StaticType::build_primitive(&PrimitiveType::Bool, scope)
                .map(|value| (e_static!(value))),
            Primitive::Char(_) => StaticType::build_primitive(&PrimitiveType::Char, scope)
                .map(|value| (e_static!(value))),
        }
    }
}

impl TypeOf for Slice {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for StrSlice {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        StaticType::build_str_slice(
            &StrSliceType {
                size: self.value.len() + self.padding,
            },
            scope,
        )
        .map(|value| (e_static!(value)))
    }
}

impl TypeOf for Vector {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Tuple {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}

impl TypeOf for Closure {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let expr_type = expr.type_of(&scope)?;
            params_types.push(expr_type);
        }
        let ret_type = self.scope.type_of(&scope)?;
        let mut scope_params_size = 0;
        for (v, _) in self
            .scope
            .scope()?
            .try_read()
            .map_err(|_| SemanticError::ConcurrencyError)?
            .vars()
        {
            scope_params_size += arw_read!(v, SemanticError::ConcurrencyError)?
                .type_sig
                .size_of();
        }
        StaticType::build_closure(
            &params_types,
            &ret_type,
            self.closed,
            scope_params_size,
            scope,
        )
        .map(|value| e_static!(value))
    }
}

impl TypeOf for ExprScope {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ExprScope::Scope(value) => value.type_of(&scope),
            ExprScope::Expr(value) => value.type_of(&scope),
        }
    }
}
impl TypeOf for ClosureParam {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ClosureParam::Full(var) => var.type_of(&scope),
            ClosureParam::Minimal(_) => Ok(Either::Static(
                <StaticType as BuildStaticType>::build_any().into(),
            )),
        }
    }
}
impl TypeOf for Address {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let addr_type = self.value.type_of(&scope)?;

        StaticType::build_addr_from(&addr_type, scope).map(|value| e_static!(value))
    }
}
impl TypeOf for PtrAccess {
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
impl TypeOf for Struct {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope.find_type(&self.id)?;
        user_type.type_of(&scope)
    }
}
impl TypeOf for Union {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(&scope)
    }
}

impl TypeOf for Enum {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(&scope)
    }
}
impl TypeOf for Map {
    fn type_of(&self, scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        self.metadata
            .signature()
            .ok_or(SemanticError::NotResolvedYet)
    }
}
