use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError};

use super::{
    Address, Channel, Closure, ClosureParam, ClosureScope, Data, Enum, KeyData, Map, MultiData,
    Primitive, PtrAccess, Slice, Struct, Tuple, Union, VarID, Variable, Vector,
};

impl<Scope: ScopeApi> Resolve<Scope> for Data {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Data::Primitive(value) => value.resolve(scope),
            Data::Slice(value) => value.resolve(scope),
            Data::Vec(value) => value.resolve(scope),
            Data::Closure(value) => value.resolve(scope),
            Data::Chan(value) => value.resolve(scope),
            Data::Tuple(value) => value.resolve(scope),
            Data::Address(value) => value.resolve(scope),
            Data::PtrAccess(value) => value.resolve(scope),
            Data::Variable(value) => value.resolve(scope),
            Data::Unit => Ok(()),
            Data::Map(value) => value.resolve(scope),
            Data::Struct(value) => value.resolve(scope),
            Data::Union(value) => value.resolve(scope),
            Data::Enum(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Variable {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Variable::Var(VarID(value)) => {
                let _ = scope.find_var(value)?;

                Ok(())
            }
            Variable::FieldAccess(_) => todo!(),
            Variable::ListAccess(_) => todo!(),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Primitive {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Slice {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Slice::String(value) => Ok(()),
            Slice::List(value) => match value.iter().find_map(|expr| expr.resolve(scope).err()) {
                Some(err) => Err(err),
                None => Ok(()),
            },
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Vector {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Vector::Init(value) => match value.iter().find_map(|expr| expr.resolve(scope).err()) {
                Some(err) => Err(err),
                None => Ok(()),
            },
            Vector::Def { length, capacity } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Tuple {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MultiData {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self.iter().find_map(|expr| expr.resolve(scope).err()) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Closure {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = {
            match self
                .params
                .iter()
                .find_map(|expr| expr.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        let _ = self.scope.resolve(scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureScope {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ClosureScope::Scope(value) => value.resolve(scope),
            ClosureScope::Expr(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ClosureParam {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            ClosureParam::Full { id, signature } => signature.resolve(scope),
            ClosureParam::Minimal(value) => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Address {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PtrAccess {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.0.resolve(scope)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Channel {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Channel::Receive { addr, .. } => addr.resolve(scope),
            Channel::Send { addr, msg } => {
                let _ = addr.resolve(scope)?;
                let _ = msg.resolve(scope)?;
                Ok(())
            }
            Channel::Init(id) => scope.register_chan(id),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Struct {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Struct::Inline { id, data } => {
                let user_type = scope.find_type(id)?;
                let _ = data.resolve(scope)?;
                let _ = user_type.compatible_with(data, scope)?;
                Ok(())
            }
            Struct::Field { id, fields } => {
                let user_type = scope.find_type(id)?;
                let _ = {
                    match fields
                        .iter()
                        .find_map(|(_, expr)| expr.resolve(scope).err())
                    {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                }?;
                let _ = user_type.compatible_with(fields, scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Union {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Union::Inline {
                typename,
                variant,
                data,
            } => {
                let user_type = scope.find_type(typename)?;
                let _ = data.resolve(scope)?;

                let _ = user_type.compatible_with(&(variant, data), scope)?;
                Ok(())
            }
            Union::Field {
                typename,
                variant,
                fields,
            } => {
                let user_type = scope.find_type(typename)?;
                let _ = {
                    match fields
                        .iter()
                        .find_map(|(_, expr)| expr.resolve(scope).err())
                    {
                        Some(e) => Err(e),
                        None => Ok(()),
                    }
                }?;
                let _ = user_type.compatible_with(&(variant, fields), scope)?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Enum {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let user_type = scope.find_type(&self.typename)?;
        user_type.compatible_with(&(&self.typename, &self.value), scope)?;

        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Map {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Map::Init { fields } => {
                match fields
                    .iter()
                    .find_map(|(key, value)| match key.resolve(scope) {
                        Ok(_) => value.resolve(scope).err(),
                        Err(e) => Some(e),
                    }) {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            }
            Map::Def { length, capacity } => Ok(()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for KeyData {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            KeyData::Number(_) => Ok(()),
            KeyData::Bool(_) => Ok(()),
            KeyData::Char(_) => Ok(()),
            KeyData::String(_) => Ok(()),
            KeyData::Address(value) => value.resolve(scope),
            KeyData::Enum(value) => value.resolve(scope),
        }
    }
}
