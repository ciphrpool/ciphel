use super::{
    Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, StructVariant, TypeDef,
    UnionDef, UnionVariant,
};
use crate::semantic::BuildFn;
use crate::semantic::BuildType;
use crate::semantic::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::RetrieveTypeInfo;

use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

impl<Scope: ScopeApi> Resolve<Scope> for Definition {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Definition::Type(value) => value.resolve(scope, context),
            Definition::Fn(value) => value.resolve(scope, context),
            Definition::Event(value) => value.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TypeDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = match self {
            TypeDef::Struct(value) => value.resolve(scope, context),
            TypeDef::Union(value) => value.resolve(scope, context),
            TypeDef::Enum(value) => value.resolve(scope, context),
        }?;
        let _ = scope.register_type(Scope::UserType::build_type(self))?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructVariant {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            StructVariant::Fields(fields) => match fields
                .iter()
                .find_map(|(_, field)| field.resolve(scope, context).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            },
            StructVariant::Inline(values) => values.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.fields.resolve(scope, context)?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionVariant {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            UnionVariant::Id => Ok(()),
            UnionVariant::Fields(fields) => match fields
                .iter()
                .find_map(|(_, field)| field.resolve(scope, context).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            },
            UnionVariant::Inline(values) => values.resolve(scope, context),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = {
            match self
                .variants
                .iter()
                .find_map(|(_, variant)| variant.resolve(scope, context).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EnumDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = {
            match self
                .params
                .iter()
                .find_map(|value| value.resolve(scope, context).err())
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }?;
        let _ = self.ret.resolve(scope, context)?;

        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(self.params.iter().map(|param| {
            let param_type = param.type_of(scope).unwrap_or(None);
            let id = &param.id;
            Scope::Var::build_var(id, &param_type.unwrap())
        }));
        let _ = self.scope.resolve(&inner_scope, &None)?;

        let return_type = self.ret.type_of(scope)?;
        let _ = return_type.compatible_with(&self.scope, scope)?;

        let _ = scope.register_fn(Scope::Fn::build_fn(self))?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventDef {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventCondition {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
