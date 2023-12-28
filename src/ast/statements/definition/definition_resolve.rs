use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{
    Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, StructVariant, UnionDef,
    UnionVariant,
};

impl<Scope: ScopeApi> Resolve<Scope> for Definition {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Definition::Struct(value) => value.resolve(scope),
            Definition::Union(value) => value.resolve(scope),
            Definition::Enum(value) => value.resolve(scope),
            Definition::Fn(value) => value.resolve(scope),
            Definition::Event(value) => value.resolve(scope),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructVariant {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            StructVariant::Fields(fields) => match fields
                .iter()
                .find_map(|(_, field)| field.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            },
            StructVariant::Inline(values) => values.resolve(scope),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructDef {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.fields.resolve(scope)?;
        let _ = scope.register_type(todo!())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionVariant {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            UnionVariant::Id => Ok(()),
            UnionVariant::Fields(fields) => match fields
                .iter()
                .find_map(|(_, field)| field.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            },
            UnionVariant::Inline(values) => values.resolve(scope),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionDef {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = {
            match self
                .variants
                .iter()
                .find_map(|(_, variant)| variant.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        let _ = scope.register_type(todo!())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EnumDef {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = scope.register_type(todo!())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnDef {
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
                .find_map(|value| value.resolve(scope).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        let _ = self.ret.resolve(scope)?;
        let _ = self.scope.resolve(scope)?;

        let return_type = self.ret.type_of(scope)?;
        let _ = return_type.compatible_with(&self.scope, scope)?;

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventDef {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventCondition {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
