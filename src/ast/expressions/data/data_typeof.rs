use super::{
    Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, FieldAccess, KeyData,
    ListAccess, Map, MultiData, Primitive, PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable,
    Vector,
};
use crate::ast::types::{PrimitiveType, Type};
use crate::semantic::scope::BuildStaticType;
use crate::{
    ast::{expressions::Expression, types::SliceType},
    semantic::{
        scope::{type_traits::GetSubTypes, ScopeApi},
        EitherType, Resolve, SemanticError, TypeOf,
    },
};

impl<Scope: ScopeApi> TypeOf<Scope> for Data {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Data::Primitive(value) => value.type_of(scope),
            Data::Slice(value) => value.type_of(scope),
            Data::Vec(value) => value.type_of(scope),
            Data::Closure(value) => value.type_of(scope),
            Data::Chan(value) => value.type_of(scope),
            Data::Tuple(value) => value.type_of(scope),
            Data::Address(value) => value.type_of(scope),
            Data::PtrAccess(value) => value.type_of(scope),
            Data::Variable(value) => value.type_of(scope),
            Data::Unit => {
                let result_type: Scope::StaticType = Scope::StaticType::build_unit();
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Data::Map(value) => value.type_of(scope),
            Data::Struct(value) => value.type_of(scope),
            Data::Union(value) => value.type_of(scope),
            Data::Enum(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Variable {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Variable::Var(value) => value.type_of(scope),
            Variable::FieldAccess(value) => value.type_of(scope),
            Variable::ListAccess(value) => value.type_of(scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VarID {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var = scope.find_var(&self.0)?;
        var.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FieldAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.field.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ListAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(scope)?;
        Ok(<Option<
            EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        > as GetSubTypes<Scope>>::get_item(&var_type))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for String {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let result_type: Scope::StaticType = Scope::StaticType::build_slice(&SliceType::String);
        let result_type = result_type.type_of(scope)?;
        Ok(result_type)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Primitive {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Primitive::Number(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Number);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Primitive::Float(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Float);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Primitive::Bool(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Bool);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Primitive::Char(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Char);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Slice {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Slice::String(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_slice(&SliceType::String);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Slice::List(vec) => {
                let mut list_types = Vec::with_capacity(vec.len());
                for expr in vec {
                    let Some(expr_type) = expr.type_of(scope)? else {
                        return Err(SemanticError::CantInferType);
                    };
                    list_types.push(expr_type);
                }
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_slice_from(&list_types);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Vector {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Vector::Init(vec) => {
                let Some(expr_type) = vec.first().map(|expr| expr.type_of(scope)) else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(expr_type) = expr_type? else {
                    return Err(SemanticError::CantInferType);
                };
                let result_type: Scope::StaticType = Scope::StaticType::build_vec_from(&expr_type);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Vector::Def { .. } => {
                let any_type: Scope::StaticType = Scope::StaticType::build_any();
                let Some(any_type) = any_type.type_of(scope)? else {
                    return Err(SemanticError::CantInferType);
                };

                let result_type: Scope::StaticType = Scope::StaticType::build_vec_from(&any_type);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Tuple {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut list_types = Vec::with_capacity(self.0.len());
        for expr in &self.0 {
            let Some(expr_type) = expr.type_of(scope)? else {
                return Err(SemanticError::CantInferType);
            };
            list_types.push(expr_type);
        }
        let result_type: Scope::StaticType = Scope::StaticType::build_tuple_from(&list_types);
        let result_type = result_type.type_of(scope)?;
        Ok(result_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Closure {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let Some(expr_type) = expr.type_of(scope)? else {
                return Err(SemanticError::CantInferType);
            };
            params_types.push(expr_type);
        }
        let Some(ret_type) = self.scope.type_of(scope)? else {
            return Err(SemanticError::CantInferType);
        };
        let result_type: Scope::StaticType =
            Scope::StaticType::build_fn_from(&params_types, &ret_type);
        let result_type = result_type.type_of(scope)?;
        Ok(result_type)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureScope {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ClosureScope::Scope(value) => value.type_of(scope),
            ClosureScope::Expr(value) => value.type_of(scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureParam {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ClosureParam::Full { id, signature } => signature.type_of(scope),
            ClosureParam::Minimal(_) => Ok(None),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Address {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let Some(addr_type) = self.0.type_of(scope)? else {
            return Err(SemanticError::CantInferType);
        };

        let result_type: Scope::StaticType = Scope::StaticType::build_addr_from(&addr_type);
        let result_type = result_type.type_of(scope)?;
        Ok(result_type)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PtrAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let Some(addr_type) = self.0.type_of(scope)? else {
            return Err(SemanticError::CantInferType);
        };

        let result_type: Scope::StaticType = Scope::StaticType::build_ptr_access_from(&addr_type);
        let result_type = result_type.type_of(scope)?;
        Ok(result_type)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Channel {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Channel::Receive { addr, timeout } => {
                let addr_type = addr.type_of(scope)?;
                let msg_type = <Option<
                    EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
                > as GetSubTypes<Scope>>::get_return(&addr_type);
                let result_type = msg_type.type_of(scope)?;
                Ok(result_type)
            }
            Channel::Send { addr, msg } => {
                let unit_type: Scope::StaticType = Scope::StaticType::build_unit();
                let result_type = unit_type.type_of(scope)?;
                Ok(result_type)
            }
            Channel::Init(_) => {
                let any_type: Scope::StaticType = Scope::StaticType::build_any();
                let Some(any_type) = any_type.type_of(scope)? else {
                    return Err(SemanticError::CantInferType);
                };

                let result_type: Scope::StaticType = Scope::StaticType::build_chan_from(&any_type);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Struct {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let id = match self {
            Struct::Inline { id, .. } => id,
            Struct::Field { id, .. } => id,
        };
        let user_type = scope.find_type(id)?;
        user_type.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Union {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let typename = match self {
            Union::Inline { typename, .. } => typename,
            Union::Field { typename, .. } => typename,
        };
        let user_type = scope.find_type(typename)?;
        user_type.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Enum {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Map {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Map::Init { fields } => {
                let Some((key, value)) = fields.first() else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(key_type) = key.type_of(scope)? else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(value_type) = value.type_of(scope)? else {
                    return Err(SemanticError::CantInferType);
                };

                let result_type: Scope::StaticType =
                    Scope::StaticType::build_map_from(&key_type, &value_type);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            Map::Def { length, capacity } => {
                let result_type: Scope::StaticType = Scope::StaticType::build_any();
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for KeyData {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            KeyData::Number(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Number);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            KeyData::Bool(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Bool);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            KeyData::Char(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_primitive(&PrimitiveType::Char);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            KeyData::String(_) => {
                let result_type: Scope::StaticType =
                    Scope::StaticType::build_slice(&SliceType::String);
                let result_type = result_type.type_of(scope)?;
                Ok(result_type)
            }
            KeyData::Address(value) => value.type_of(scope),
            KeyData::Enum(value) => value.type_of(scope),
        }
    }
}
