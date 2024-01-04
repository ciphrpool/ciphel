

use {
    crate::semantic::{CompatibleWith, EitherType, MergeType, SemanticError, TypeOf},
};

use crate::ast;

use super::{
    type_traits::{GetSubTypes, OperandMerging, TypeChecking},
    user_type_impl::{Enum, UserType},
    BuildStaticType, ScopeApi,
};

type SubType = Box<EitherType<UserType, StaticType>>;

#[derive(Debug, Clone, PartialEq)]
pub enum StaticType {
    Primitive(PrimitiveType),
    Slice(SliceType),
    Vec(VecType),
    Fn(FnType),
    Chan(ChanType),
    Tuple(TupleType),
    Unit,
    Any,
    Error,
    Address(AddrType),
    Map(MapType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Number,
    Float,
    Char,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SliceType {
    String,
    List(usize, SubType),
}
#[derive(Debug, Clone, PartialEq)]
pub struct VecType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct FnType {
    pub params: Types,
    pub ret: SubType,
}
pub type Types = Vec<EitherType<UserType, StaticType>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ChanType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType(pub Types);

#[derive(Debug, Clone, PartialEq)]
pub struct AddrType(pub SubType);

#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub keys_type: KeyType,
    pub values_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    Primitive(PrimitiveType),
    Address(AddrType),
    Slice(SliceType),
    Enum(Enum),
}

impl<Scope: ScopeApi<StaticType = Self>> TypeOf<Scope> for StaticType {
    fn type_of(
        &self,
        _scope: &Scope,
    ) -> Result<EitherType<<Scope as ScopeApi>::UserType, Self>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized,
    {
        Ok(EitherType::Static(self.clone()))
    }
}

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> CompatibleWith<Scope> for StaticType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Scope) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            StaticType::Primitive(value) => value.compatible_with(other, scope),
            StaticType::Slice(value) => value.compatible_with(other, scope),
            StaticType::Vec(value) => value.compatible_with(other, scope),
            StaticType::Fn(value) => value.compatible_with(other, scope),
            StaticType::Chan(value) => value.compatible_with(other, scope),
            StaticType::Tuple(value) => value.compatible_with(other, scope),
            StaticType::Unit => {
                let other_type = other.type_of(scope)?;
                if let EitherType::Static(StaticType::Unit) = other_type {
                    return Ok(());
                } else {
                    return Err(SemanticError::IncompatibleTypes);
                }
            }
            StaticType::Address(value) => value.compatible_with(other, scope),
            StaticType::Map(value) => value.compatible_with(other, scope),
            StaticType::Any => Ok(()),
            StaticType::Error => Ok(()),
        }
    }
}

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> BuildStaticType<Scope>
    for StaticType
{
    fn build_primitive(
        type_sig: &ast::types::PrimitiveType,
        _scope: &Scope,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Primitive(match type_sig {
            ast::types::PrimitiveType::Number => PrimitiveType::Number,
            ast::types::PrimitiveType::Float => PrimitiveType::Float,
            ast::types::PrimitiveType::Char => PrimitiveType::Char,
            ast::types::PrimitiveType::Bool => PrimitiveType::Bool,
        }))
    }

    fn build_slice(type_sig: &ast::types::SliceType, scope: &Scope) -> Result<Self, SemanticError> {
        match type_sig {
            ast::types::SliceType::String => Ok(Self::Slice(SliceType::String)),
            ast::types::SliceType::List(size, inner) => {
                let inner = inner.type_of(scope)?;
                Ok(Self::Slice(SliceType::List(size.clone(), Box::new(inner))))
            }
        }
    }

    fn build_slice_from(
        size: &usize,
        type_sig: &EitherType<Scope::UserType, Scope::StaticType>,
        _scope: &Scope,
    ) -> Result<Self, SemanticError> {
        Ok(Self::Slice(SliceType::List(
            size.clone(),
            Box::new(type_sig.clone()),
        )))
    }

    fn build_tuple(type_sig: &ast::types::TupleType, scope: &Scope) -> Result<Self, SemanticError> {
        let mut vec = Vec::with_capacity(type_sig.0.len());
        for subtype in &type_sig.0 {
            let subtype = subtype.type_of(scope)?;
            vec.push(subtype);
        }
        Ok(Self::Tuple(TupleType(vec)))
    }

    fn build_tuple_from(
        type_sig: &Vec<EitherType<<Scope as ScopeApi>::UserType, Self>>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let mut vec = Vec::with_capacity(type_sig.len());
        for subtype in type_sig {
            let subtype = subtype.type_of(scope)?;
            vec.push(subtype);
        }
        Ok(Self::Tuple(TupleType(vec)))
    }

    fn build_vec(type_sig: &ast::types::VecType, scope: &Scope) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_vec_from(
        type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(scope)?;
        Ok(Self::Vec(VecType(Box::new(subtype))))
    }

    fn build_error() -> Self {
        Self::Error
    }

    fn build_fn(type_sig: &ast::types::FnType, scope: &Scope) -> Result<Self, SemanticError> {
        let mut params = Vec::with_capacity(type_sig.params.len());
        for subtype in &type_sig.params {
            let subtype = subtype.type_of(scope)?;
            params.push(subtype);
        }
        let ret_type = type_sig.ret.type_of(scope)?;
        Ok(Self::Fn(FnType {
            params,
            ret: Box::new(ret_type),
        }))
    }

    fn build_fn_from(
        params: &Vec<EitherType<<Scope as ScopeApi>::UserType, Self>>,
        ret: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let mut out_params = Vec::with_capacity(params.len());
        for subtype in params {
            let subtype = subtype.type_of(scope)?;
            out_params.push(subtype);
        }
        let ret_type = ret.type_of(scope)?;
        Ok(Self::Fn(FnType {
            params: out_params,
            ret: Box::new(ret_type),
        }))
    }

    fn build_chan(type_sig: &ast::types::ChanType, scope: &Scope) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(scope)?;
        Ok(Self::Chan(ChanType(Box::new(subtype))))
    }

    fn build_chan_from(
        type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(scope)?;
        Ok(Self::Chan(ChanType(Box::new(subtype))))
    }

    fn build_unit() -> Self {
        Self::Unit
    }

    fn build_any() -> Self {
        Self::Any
    }

    fn build_addr(type_sig: &ast::types::AddrType, scope: &Scope) -> Result<Self, SemanticError> {
        let subtype = type_sig.0.type_of(scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_addr_from(
        type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let subtype = type_sig.type_of(scope)?;
        Ok(Self::Address(AddrType(Box::new(subtype))))
    }

    fn build_ptr_access_from(
        _type_sig: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        _scope: &Scope,
    ) -> Result<Self, SemanticError> {
        todo!()
    }

    fn build_map(type_sig: &ast::types::MapType, scope: &Scope) -> Result<Self, SemanticError> {
        let key_type = {
            match &type_sig.keys_type {
                ast::types::KeyType::Primitive(value) => KeyType::Primitive(match value {
                    ast::types::PrimitiveType::Number => PrimitiveType::Number,
                    ast::types::PrimitiveType::Float => PrimitiveType::Float,
                    ast::types::PrimitiveType::Char => PrimitiveType::Char,
                    ast::types::PrimitiveType::Bool => PrimitiveType::Bool,
                }),
                ast::types::KeyType::Address(value) => {
                    let subtype = value.0.type_of(scope)?;
                    KeyType::Address(AddrType(Box::new(subtype)))
                }
                ast::types::KeyType::Slice(value) => match value {
                    ast::types::SliceType::String => KeyType::Slice(SliceType::String),
                    ast::types::SliceType::List(size, inner) => {
                        let inner = inner.type_of(scope)?;
                        KeyType::Slice(SliceType::List(size.clone(), Box::new(inner)))
                    }
                },
                ast::types::KeyType::EnumID(value) => {
                    let UserType::Enum(enum_type) = scope.find_type(&value)? else {
                        return Err(SemanticError::IncompatibleTypes);
                    };
                    KeyType::Enum(enum_type.clone())
                }
            }
        };

        let subtype = type_sig.type_of(scope)?;
        Ok(Self::Map(MapType {
            keys_type: key_type,
            values_type: Box::new(subtype),
        }))
    }

    fn build_map_from(
        key: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        value: &EitherType<<Scope as ScopeApi>::UserType, Self>,
        scope: &Scope,
    ) -> Result<Self, SemanticError> {
        let key_type = {
            match key {
                EitherType::Static(value) => match value {
                    StaticType::Primitive(value) => KeyType::Primitive(value.clone()),
                    StaticType::Slice(value) => KeyType::Slice(value.clone()),
                    StaticType::Address(value) => KeyType::Address(value.clone()),
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
                EitherType::User(value) => match value {
                    UserType::Enum(value) => KeyType::Enum(value.clone()),
                    _ => return Err(SemanticError::IncompatibleTypes),
                },
            }
        };

        let subtype = value.type_of(scope)?;
        Ok(Self::Map(MapType {
            keys_type: key_type,
            values_type: Box::new(subtype),
        }))
    }
}

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> GetSubTypes<Scope> for StaticType {
    fn get_nth(&self, n: &usize) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::Fn(value) => value.params.get(*n).map(|v| v.clone()),
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => value.0.get(*n).map(|v| v.clone()),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(_) => None,
            StaticType::Map(_) => None,
        }
    }

    fn get_item(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(value) => match value {
                SliceType::String => Some(EitherType::Static(Self::Primitive(PrimitiveType::Char))),
                SliceType::List(_, value) => Some(value.as_ref().clone()),
            },
            StaticType::Vec(value) => Some(value.0.as_ref().clone()),
            StaticType::Fn(_) => None,
            StaticType::Chan(value) => Some(value.0.as_ref().clone()),
            StaticType::Tuple(_) => None,
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(_) => None,
            StaticType::Map(value) => Some(value.values_type.as_ref().clone()),
        }
    }
    fn get_return(&self) -> Option<EitherType<Scope::UserType, Scope::StaticType>> {
        if let StaticType::Fn(value) = self {
            Some(value.ret.as_ref().clone())
        } else {
            None
        }
    }
    fn get_fields(
        &self,
    ) -> Option<
        Vec<(
            Option<String>,
            EitherType<Scope::UserType, Scope::StaticType>,
        )>,
    > {
        match self {
            StaticType::Primitive(_) => None,
            StaticType::Slice(_) => None,
            StaticType::Vec(_) => None,
            StaticType::Fn(value) => Some(
                value
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
            StaticType::Chan(_) => None,
            StaticType::Tuple(value) => Some(
                value
                    .0
                    .clone()
                    .into_iter()
                    .map(|param| (None, param))
                    .collect(),
            ),
            StaticType::Unit => None,
            StaticType::Any => None,
            StaticType::Error => None,
            StaticType::Address(_) => None,
            StaticType::Map(_) => None,
        }
    }
}

impl<Scope: ScopeApi<StaticType = Self>> TypeChecking<Scope> for StaticType {
    fn is_iterable(&self) -> bool {
        match self {
            StaticType::Primitive(_) => false,
            StaticType::Slice(_) => true,
            StaticType::Vec(_) => true,
            StaticType::Fn(_) => false,
            StaticType::Chan(_) => true,
            StaticType::Tuple(_) => false,
            StaticType::Unit => false,
            StaticType::Any => false,
            StaticType::Error => false,
            StaticType::Address(_) => false,
            StaticType::Map(_) => true,
        }
    }
    fn is_channel(&self) -> bool {
        if let StaticType::Chan(_) = self {
            true
        } else {
            false
        }
    }
    fn is_boolean(&self) -> bool {
        if let StaticType::Primitive(PrimitiveType::Bool) = self {
            true
        } else {
            false
        }
    }

    fn is_callable(&self) -> bool {
        if let StaticType::Fn(_) = self {
            true
        } else {
            false
        }
    }
    fn is_any(&self) -> bool {
        if let StaticType::Any = self {
            true
        } else {
            false
        }
    }
}

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> OperandMerging<Scope> for StaticType {
    fn can_minus(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EitherType<UserType, StaticType> as OperandMerging<Scope>>::can_minus(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn can_negate(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EitherType<UserType, StaticType> as OperandMerging<Scope>>::can_negate(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn can_high_ord_math(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_high_ord_math(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => Ok(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Number),
                    )),
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => {
                <EitherType<UserType, StaticType> as OperandMerging<Scope>>::merge_high_ord_math(
                    &value.0, other, scope,
                )
            }
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_low_ord_math(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_low_ord_math(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::merge_low_ord_math(&value.0, other, scope),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_shift(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => {
                <EitherType<UserType, StaticType> as OperandMerging<Scope>>::can_shift(&value.0)
            }
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::merge_shift(&value.0, other, scope),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_and(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_bitwise_and(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::merge_bitwise_and(&value.0, other, scope),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_xor(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_bitwise_xor(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::merge_bitwise_xor(&value.0, other, scope),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_bitwise_or(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => Ok(()),
                PrimitiveType::Float => Ok(()),
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperation),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_bitwise_or(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
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
        let other_type = other.type_of(scope)?;
        let EitherType::Static(other_type) = other_type else {
            return Err(SemanticError::IncompatibleOperands);
        };
        if let StaticType::Error = other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Number => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Float => match other_type {
                    StaticType::Primitive(PrimitiveType::Number) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    StaticType::Primitive(PrimitiveType::Float) => {
                        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                    }
                    _ => Err(SemanticError::IncompatibleOperands),
                },
                PrimitiveType::Char => Err(SemanticError::IncompatibleOperands),
                PrimitiveType::Bool => Err(SemanticError::IncompatibleOperands),
            },
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::merge_bitwise_xor(&value.0, other, scope),
            _ => Err(SemanticError::IncompatibleOperands),
        }
    }

    fn can_comparaison(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(_) => Ok(()),
            StaticType::Slice(_) => Ok(()),
            StaticType::Vec(_) => Ok(()),
            StaticType::Fn(_) => Err(SemanticError::IncompatibleOperation),
            StaticType::Chan(_) => Err(SemanticError::IncompatibleOperation),
            StaticType::Tuple(_) => Ok(()),
            StaticType::Unit => Ok(()),
            StaticType::Any => Err(SemanticError::IncompatibleOperation),
            StaticType::Error => todo!(),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_comparaison(&value.0),
            StaticType::Map(_) => Ok(()),
        }
    }
    fn merge_comparaison<Other>(
        &self,
        _other: &Other,
        _scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Bool)))
    }

    fn can_logical_and(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_bitwise_or(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_and<Other>(
        &self,
        _other: &Other,
        _scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Bool)))
    }

    fn can_logical_or(&self) -> Result<(), SemanticError> {
        match self {
            StaticType::Primitive(value) => match value {
                PrimitiveType::Bool => Ok(()),
                _ => Err(SemanticError::IncompatibleOperation),
            },
            StaticType::Error => Ok(()),
            StaticType::Address(value) => <EitherType<UserType, StaticType> as OperandMerging<
                Scope,
            >>::can_bitwise_or(&value.0),
            _ => Err(SemanticError::IncompatibleOperation),
        }
    }
    fn merge_logical_or<Other>(
        &self,
        _other: &Other,
        _scope: &Scope,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Bool)))
    }
}

impl<Scope: ScopeApi<StaticType = Self, UserType = UserType>> MergeType<Scope> for StaticType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<EitherType<<Scope as ScopeApi>::UserType, Self>, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(static_other_type) = &other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if let StaticType::Error = static_other_type {
            return Ok(EitherType::Static(StaticType::Error));
        }
        match self {
            StaticType::Primitive(value) => value.merge(other, scope),
            StaticType::Slice(value) => value.merge(other, scope),
            StaticType::Vec(value) => value.merge(other, scope),
            StaticType::Fn(value) => value.merge(other, scope),
            StaticType::Chan(value) => value.merge(other, scope),
            StaticType::Tuple(value) => value.merge(other, scope),
            StaticType::Unit => Ok(other_type),
            StaticType::Any => Err(SemanticError::CantInferType),
            StaticType::Error => Ok(EitherType::Static(StaticType::Error)),
            StaticType::Address(value) => value.merge(other, scope),
            StaticType::Map(value) => value.merge(other, scope),
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope>
    for PrimitiveType
{
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Primitive(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            PrimitiveType::Number => match other_type {
                PrimitiveType::Number => {
                    Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Number)))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            PrimitiveType::Float => match other_type {
                PrimitiveType::Number => {
                    Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                }
                PrimitiveType::Float => {
                    Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Float)))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            PrimitiveType::Char => match other_type {
                PrimitiveType::Char => {
                    Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Char)))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            PrimitiveType::Bool => match other_type {
                PrimitiveType::Bool => {
                    Ok(EitherType::Static(StaticType::Primitive(PrimitiveType::Bool)))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for SliceType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Slice(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            SliceType::String => match other_type {
                SliceType::String => Ok(EitherType::Static(StaticType::Slice(SliceType::String))),
                SliceType::List(_, _) => Err(SemanticError::IncompatibleTypes),
            },
            SliceType::List(size, subtype) => match other_type {
                SliceType::String => Err(SemanticError::IncompatibleTypes),
                SliceType::List(other_size, other_subtype) => {
                    if size != &other_size {
                        Err(SemanticError::IncompatibleTypes)
                    } else {
                        let merged = subtype.merge(other_subtype.as_ref(), scope)?;
                        Ok(EitherType::Static(StaticType::Slice(SliceType::List(
                            size.clone(),
                            Box::new(merged),
                        ))))
                    }
                }
            },
        }
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for VecType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Vec(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(EitherType::Static(StaticType::Vec(VecType(Box::new(merged)))))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for FnType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Fn(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.params.len() != other_type.params.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        let mut merged_params = Vec::with_capacity(self.params.len());
        for (self_inner, other_inner) in self.params.iter().zip(other_type.params.iter()) {
            let merged = self_inner.merge(other_inner, scope)?;
            merged_params.push(merged);
        }
        let merged_ret = self.ret.merge(other_type.ret.as_ref(), scope)?;
        Ok(EitherType::Static(StaticType::Fn(FnType {
            params: merged_params,
            ret: Box::new(merged_ret),
        })))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for ChanType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Chan(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(EitherType::Static(StaticType::Chan(ChanType(Box::new(merged)))))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for TupleType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Tuple(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.0.len() != other_type.0.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        let mut merged_vec = Vec::with_capacity(self.0.len());
        for (self_inner, other_inner) in self.0.iter().zip(other_type.0.iter()) {
            let merged = self_inner.merge(other_inner, scope)? else {
                return Err(SemanticError::CantInferType);
            };
            merged_vec.push(merged);
        }
        Ok(EitherType::Static(StaticType::Tuple(TupleType(merged_vec))))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for AddrType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Address(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let merged = self.0.merge(other_type.0.as_ref(), scope)?;
        Ok(EitherType::Static(StaticType::Address(AddrType(Box::new(merged)))))
    }
}

impl<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>> MergeType<Scope> for MapType {
    fn merge<Other>(
        &self,
        other: &Other,
        scope: &Scope,
    ) -> Result<
        EitherType<<Scope as ScopeApi>::UserType, <Scope as ScopeApi>::StaticType>,
        SemanticError,
    >
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(scope)?;
        let EitherType::Static(StaticType::Map(other_type)) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };

        let merged_key = self.keys_type.merge_key(&other_type.keys_type, scope)?;

        let merged_value = self
            .values_type
            .merge(other_type.values_type.as_ref(), scope)?;
        Ok(EitherType::Static(StaticType::Map(MapType {
            keys_type: merged_key,
            values_type: Box::new(merged_value),
        })))
    }
}

impl KeyType {
    fn merge_key<Scope: ScopeApi<StaticType = StaticType, UserType = UserType>>(
        &self,
        other: &KeyType,
        scope: &Scope,
    ) -> Result<KeyType, SemanticError> {
        match (self, other) {
            (KeyType::Primitive(value), KeyType::Primitive(other_value)) => {
                match (value, other_value) {
                    (PrimitiveType::Number, PrimitiveType::Number) => {
                        Ok(KeyType::Primitive(PrimitiveType::Number))
                    }
                    (PrimitiveType::Number, PrimitiveType::Float) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Float, PrimitiveType::Number) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Float, PrimitiveType::Float) => {
                        Ok(KeyType::Primitive(PrimitiveType::Float))
                    }
                    (PrimitiveType::Char, PrimitiveType::Char) => {
                        Ok(KeyType::Primitive(PrimitiveType::Char))
                    }
                    (PrimitiveType::Bool, PrimitiveType::Bool) => {
                        Ok(KeyType::Primitive(PrimitiveType::Bool))
                    }
                    _ => Err(SemanticError::IncompatibleTypes),
                }
            }
            (KeyType::Address(value), KeyType::Address(other_value)) => {
                let merged = value.0.merge(other_value.0.as_ref(), scope)?;
                Ok(KeyType::Address(AddrType(Box::new(merged))))
            }
            (KeyType::Slice(value), KeyType::Slice(other_value)) => match (value, other_value) {
                (SliceType::String, SliceType::String) => Ok(KeyType::Slice(SliceType::String)),
                (SliceType::List(size, subtype), SliceType::List(other_size, other_subtype)) => {
                    if size != other_size {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    let merged = subtype.merge(other_subtype.as_ref(), scope)?;
                    Ok(KeyType::Slice(SliceType::List(
                        size.clone(),
                        Box::new(merged),
                    )))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            (KeyType::Enum(value), KeyType::Enum(other_value)) => {
                let merged = value.merge(other_value)?;
                Ok(KeyType::Enum(merged))
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}
