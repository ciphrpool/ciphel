use super::{
    Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, FieldAccess, KeyData,
    ListAccess, Map, Primitive, PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable, Vector,
};
use crate::ast::types::PrimitiveType;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::MergeType;
use crate::{
    ast::types::SliceType,
    semantic::{
        scope::{type_traits::GetSubTypes, ScopeApi},
        EitherType, Resolve, SemanticError, TypeOf,
    },
};

impl<Scope: ScopeApi> TypeOf<Scope> for Data {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
            Data::Unit => Ok(EitherType::Static(Scope::StaticType::build_unit())),
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(scope)?;
        let Some(res) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_item(&var_type) else {
            return Err(SemanticError::CantInferType);
        };
        Ok(res)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for String {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        Scope::StaticType::build_slice(&SliceType::String, scope)
            .map(|value| (EitherType::Static(value)))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Primitive {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Primitive::Number(_) => {
                Scope::StaticType::build_primitive(&PrimitiveType::Number, scope)
                    .map(|value| (EitherType::Static(value)))
            }
            Primitive::Float(_) => Scope::StaticType::build_primitive(&PrimitiveType::Float, scope)
                .map(|value| (EitherType::Static(value))),
            Primitive::Bool(_) => Scope::StaticType::build_primitive(&PrimitiveType::Bool, scope)
                .map(|value| (EitherType::Static(value))),
            Primitive::Char(_) => Scope::StaticType::build_primitive(&PrimitiveType::Char, scope)
                .map(|value| (EitherType::Static(value))),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Slice {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Slice::String(_) => Scope::StaticType::build_slice(&SliceType::String, scope)
                .map(|value| (EitherType::Static(value))),
            Slice::List(vec) => {
                let mut list_type = EitherType::Static(Scope::StaticType::build_unit());
                for expr in vec {
                    let expr_type = expr.type_of(scope)?;
                    list_type = list_type.merge(&expr_type, scope)?;
                }

                Scope::StaticType::build_slice_from(&vec.len(), &list_type, scope)
                    .map(|value| (EitherType::Static(value)))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Vector {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Vector::Init(vec) => {
                let Some(expr_type) = vec.first().map(|expr| expr.type_of(scope)) else {
                    return Err(SemanticError::CantInferType);
                };
                let expr_type = expr_type?;

                Scope::StaticType::build_vec_from(&expr_type, scope)
                    .map(|value| EitherType::Static(value))
            }
            Vector::Def { .. } => {
                let any_type: Scope::StaticType = Scope::StaticType::build_any();
                Scope::StaticType::build_vec_from(&EitherType::Static(any_type), scope)
                    .map(|value| EitherType::Static(value))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Tuple {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut list_types = Vec::with_capacity(self.0.len());
        for expr in &self.0 {
            let expr_type = expr.type_of(scope)?;
            list_types.push(expr_type);
        }

        Scope::StaticType::build_tuple_from(&list_types, scope)
            .map(|value| EitherType::Static(value))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Closure {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let expr_type = expr.type_of(scope)?;
            params_types.push(expr_type);
        }
        let ret_type = self.scope.type_of(scope)?;

        Scope::StaticType::build_fn_from(&params_types, &ret_type, scope)
            .map(|value| EitherType::Static(value))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureScope {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ClosureParam::Full { id: _, signature } => signature.type_of(scope),
            ClosureParam::Minimal(_) => Ok(EitherType::Static(Scope::StaticType::build_any())),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Address {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let addr_type = self.0.type_of(scope)?;

        Scope::StaticType::build_addr_from(&addr_type, scope).map(|value| EitherType::Static(value))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PtrAccess {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let addr_type = self.0.type_of(scope)?;
        Scope::StaticType::build_ptr_access_from(&addr_type, scope)
            .map(|value| EitherType::Static(value))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Channel {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Channel::Receive { addr, timeout: _ } => {
                let addr_type = addr.type_of(scope)?;
                let msg_type = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_return(&addr_type);
                let Some(msg_type) = msg_type else {
                    return Err(SemanticError::CantInferType);
                };
                let result_type = msg_type.type_of(scope)?;
                Ok(result_type)
            }
            Channel::Send { addr: _, msg: _ } => {
                Ok(EitherType::Static(Scope::StaticType::build_unit()))
            }
            Channel::Init(_) => {
                let any_type: Scope::StaticType = Scope::StaticType::build_any();

                Scope::StaticType::build_chan_from(&EitherType::Static(any_type), scope)
                    .map(|value| EitherType::Static(value))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Struct {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.id)?;
        user_type.type_of(scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Union {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Enum {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Map::Init { fields } => {
                let Some((key, value)) = fields.first() else {
                    return Err(SemanticError::CantInferType);
                };
                let key_type = key.type_of(scope)?;
                let value_type = value.type_of(scope)?;

                Scope::StaticType::build_map_from(&key_type, &value_type, scope)
                    .map(|value| EitherType::Static(value))
            }
            Map::Def {
                length: _,
                capacity: _,
            } => Ok(EitherType::Static(Scope::StaticType::build_any())),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for KeyData {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            KeyData::Number(_) => Scope::StaticType::build_primitive(&PrimitiveType::Number, scope)
                .map(|value| (EitherType::Static(value))),
            KeyData::Bool(_) => Scope::StaticType::build_primitive(&PrimitiveType::Bool, scope)
                .map(|value| (EitherType::Static(value))),
            KeyData::Char(_) => Scope::StaticType::build_primitive(&PrimitiveType::Char, scope)
                .map(|value| (EitherType::Static(value))),
            KeyData::String(_) => Scope::StaticType::build_slice(&SliceType::String, scope)
                .map(|value| (EitherType::Static(value))),
            KeyData::Address(value) => value.type_of(scope),
            KeyData::Enum(value) => value.type_of(scope),
        }
    }
}
