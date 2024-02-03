use std::cell::Ref;

use super::{
    Address, Channel, Closure, ClosureParam, Data, Enum, ExprScope, FieldAccess, KeyData,
    ListAccess, Map, NumAccess, Number, Primitive, PtrAccess, Slice, StringData, Struct, Tuple,
    Union, VarID, Variable, Vector,
};
use crate::ast::types::{NumberType, PrimitiveType, StringType};

use crate::semantic::scope::static_types::StaticType;

use crate::semantic::scope::user_type_impl::UserType;

use crate::semantic::scope::BuildStaticType;
use crate::semantic::MergeType;
use crate::{
    ast::types::SliceType,
    semantic::{
        scope::{type_traits::GetSubTypes, ScopeApi},
        Either, Resolve, SemanticError, TypeOf,
    },
};

impl<Scope: ScopeApi> TypeOf<Scope> for Data<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Data::Primitive(value) => value.type_of(&scope),
            Data::Slice(value) => value.type_of(&scope),
            Data::Vec(value) => value.type_of(&scope),
            Data::Closure(value) => value.type_of(&scope),
            Data::Chan(value) => value.type_of(&scope),
            Data::Tuple(value) => value.type_of(&scope),
            Data::Address(value) => value.type_of(&scope),
            Data::PtrAccess(value) => value.type_of(&scope),
            Data::Variable(value) => value.type_of(&scope),
            Data::Unit => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Data::Map(value) => value.type_of(&scope),
            Data::Struct(value) => value.type_of(&scope),
            Data::Union(value) => value.type_of(&scope),
            Data::Enum(value) => value.type_of(&scope),
            Data::String(value) => value.type_of(&scope),
        }
    }
}

impl<InnerScope: ScopeApi> Variable<InnerScope> {
    fn typeof_based<Scope>(
        &self,
        context: &Either<UserType, StaticType>,
    ) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(VarID {
                id: value,
                metadata: _,
            }) => <Either<UserType, StaticType> as GetSubTypes>::get_field(context, value)
                .ok_or(SemanticError::UnknownField),
            Variable::FieldAccess(FieldAccess {
                var,
                field,
                metadata: _,
            }) => {
                let var_type = var.typeof_based::<Scope>(context)?;
                field.typeof_based::<Scope>(&var_type)
            }
            Variable::ListAccess(ListAccess { var, .. }) => {
                let var_type = var.typeof_based::<Scope>(context)?;
                <Either<UserType, StaticType> as GetSubTypes>::get_item(&var_type)
                    .ok_or(SemanticError::ExpectedIterable)
            }
            Variable::NumAccess(NumAccess { var, index, .. }) => {
                let var_type = var.typeof_based::<Scope>(context)?;
                <Either<UserType, StaticType> as GetSubTypes>::get_nth(&var_type, index)
                    .ok_or(SemanticError::ExpectedIndexable)
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Variable<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Variable::Var(value) => value.type_of(&scope),
            Variable::FieldAccess(value) => value.type_of(&scope),
            Variable::ListAccess(value) => value.type_of(&scope),
            Variable::NumAccess(value) => value.type_of(&scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for VarID {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var = scope.find_var(&self.id)?;
        var.type_of(&scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for FieldAccess<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(&scope)?;
        self.field.typeof_based::<Scope>(&var_type)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for NumAccess<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(&scope)?;

        <Either<UserType, StaticType> as GetSubTypes>::get_nth(&var_type, &self.index)
            .ok_or(SemanticError::ExpectedIndexable)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for ListAccess<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let var_type = self.var.type_of(&scope)?;

        <Either<UserType, StaticType> as GetSubTypes>::get_item(&var_type)
            .ok_or(SemanticError::ExpectedIterable)
    }
}

// impl<Scope: ScopeApi> TypeOf<Scope> for String {
//     fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
//     where
//         Scope: ScopeApi,
//         Self: Sized,
//     {
//         StaticType::build_slice(&SliceType::String, scope)
//             .map(|value| (Either::Static(value.into())))
//     }
// }
impl<Scope: ScopeApi> TypeOf<Scope> for Primitive {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Primitive::Number(num) => match num {
                Number::U8(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U8), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::U16(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U16), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::U32(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U32), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::U64(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U64), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::U128(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::U128), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::I8(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I8), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::I16(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I16), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::I32(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I32), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::I64(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I64), scope)
                        .map(|value| (Either::Static(value.into())))
                }
                Number::I128(_) => {
                    StaticType::build_primitive(&PrimitiveType::Number(NumberType::I128), scope)
                        .map(|value| (Either::Static(value.into())))
                }
            },
            Primitive::Float(_) => StaticType::build_primitive(&PrimitiveType::Float, scope)
                .map(|value| (Either::Static(value.into()))),
            Primitive::Bool(_) => StaticType::build_primitive(&PrimitiveType::Bool, scope)
                .map(|value| (Either::Static(value.into()))),
            Primitive::Char(_) => StaticType::build_primitive(&PrimitiveType::Char, scope)
                .map(|value| (Either::Static(value.into()))),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Slice<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut list_type =
            Either::Static(<StaticType as BuildStaticType<Scope>>::build_unit().into());
        for expr in &self.value {
            let expr_type = expr.type_of(&scope)?;
            list_type = list_type.merge(&expr_type, scope)?;
        }

        StaticType::build_slice_from(&self.value.len(), &list_type, scope)
            .map(|value| (Either::Static(value.into())))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for StringData {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        StaticType::build_string(&StringType(), scope).map(|value| (Either::Static(value.into())))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Vector<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Vector::Init {
                value: vec,
                metadata: _,
                length: _,
                capacity: _,
            } => {
                let Some(expr_type) = vec.first().map(|expr| expr.type_of(&scope)) else {
                    return Err(SemanticError::CantInferType);
                };
                let expr_type = expr_type?;

                StaticType::build_vec_from(&expr_type, scope)
                    .map(|value| Either::Static(value.into()))
            }
            Vector::Def { .. } => {
                let any_type: StaticType = <StaticType as BuildStaticType<Scope>>::build_any();
                StaticType::build_vec_from(&Either::Static(any_type.into()), scope)
                    .map(|value| Either::Static(value.into()))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Tuple<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut list_types = Vec::with_capacity(self.value.len());
        for expr in &self.value {
            let expr_type = expr.type_of(&scope)?;
            list_types.push(expr_type);
        }

        StaticType::build_tuple_from(&list_types, scope).map(|value| Either::Static(value.into()))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Closure<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let mut params_types = Vec::with_capacity(self.params.len());
        for expr in &self.params {
            let expr_type = expr.type_of(&scope)?;
            params_types.push(expr_type);
        }
        let ret_type = self.scope.type_of(&scope)?;

        StaticType::build_fn_from(&params_types, &ret_type, scope)
            .map(|value| Either::Static(value.into()))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ExprScope<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ExprScope::Scope(value) => value.type_of(&scope),
            ExprScope::Expr(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ClosureParam {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ClosureParam::Full { id: _, signature } => signature.type_of(&scope),
            ClosureParam::Minimal(_) => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_any().into(),
            )),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Address<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let addr_type = self.value.type_of(&scope)?;

        StaticType::build_addr_from(&addr_type, scope).map(|value| Either::Static(value.into()))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PtrAccess<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let _addr_type = self.value.type_of(&scope)?;
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_any().into(),
        ))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Channel<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Channel::Receive {
                addr,
                timeout: _,
                metadata: _,
            } => {
                let addr_type = addr.type_of(&scope)?;
                let msg_type =
                    <Either<UserType, StaticType> as GetSubTypes>::get_return(&addr_type);
                let Some(msg_type) = msg_type else {
                    return Err(SemanticError::CantInferType);
                };
                let result_type = msg_type.type_of(&scope)?;
                Ok(result_type)
            }
            Channel::Send {
                addr: _,
                msg: _,
                metadata: _,
            } => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_unit().into(),
            )),
            Channel::Init { .. } => {
                let any_type: StaticType = <StaticType as BuildStaticType<Scope>>::build_any();

                StaticType::build_chan_from(&Either::Static(any_type.into()), scope)
                    .map(|value| Either::Static(value.into()))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Struct<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.id)?;
        user_type.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Union<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(&scope)
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Enum {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for Map<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Map::Init {
                fields,
                metadata: _,
            } => {
                let Some((key, value)) = fields.first() else {
                    return Err(SemanticError::CantInferType);
                };
                let key_type = key.type_of(&scope)?;
                let value_type = value.type_of(&scope)?;

                StaticType::build_map_from(&key_type, &value_type, scope)
                    .map(|value| Either::Static(value.into()))
            }
            Map::Def {
                length: _,
                capacity: _,
                metadata: _,
            } => Ok(Either::Static(
                <StaticType as BuildStaticType<Scope>>::build_any().into(),
            )),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for KeyData<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<Either<UserType, StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            KeyData::Address(value) => value.type_of(&scope),
            KeyData::Enum(value) => value.type_of(&scope),
            KeyData::Primitive(value) => value.type_of(&scope),
            KeyData::String(value) => value.type_of(&scope),
        }
    }
}
