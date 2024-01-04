use super::{
    Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, StructVariant, TypeDef,
    UnionDef, UnionVariant,
};
use crate::ast::types::FnType;
use crate::ast::types::Types;
use crate::semantic::scope::BuildUserType;
use crate::semantic::scope::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, CompatibleWith, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> Resolve<Scope> for Definition {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    fn resolve(
        &self,
        scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Definition::Type(value) => value.resolve(scope, &()),
            Definition::Fn(value) => value.resolve(scope, &()),
            Definition::Event(value) => value.resolve(scope, &()),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TypeDef {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = match self {
            TypeDef::Struct(value) => value.resolve(scope, context),
            TypeDef::Union(value) => value.resolve(scope, context),
            TypeDef::Enum(value) => value.resolve(scope, context),
        }?;
        let _ = scope.register_type(Scope::UserType::build_usertype(self))?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructVariant {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        for (_, variant) in &self.variants {
            let _ = variant.resolve(scope, context)?;
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EnumDef {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        _scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        scope: &mut Scope,
        context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        for value in &self.params {
            let _ = value.resolve(scope, context)?;
        }

        let _ = self.ret.resolve(scope, context)?;
        let return_type = self.ret.type_of(scope)?;

        let mut inner_scope = scope.child_scope()?;
        inner_scope.attach(
            self.params
                .iter()
                .filter_map(|param| param.type_of(scope).ok().map(|p| (param.id.clone(), p)))
                .map(|(id, param)| Scope::Var::build_var(&id, &param)),
        );

        let _ = return_type.compatible_with(&self.scope, scope)?;
        let _ = self.scope.resolve(&mut inner_scope, &Some(return_type))?;

        // convert to FnType -> GOAL : Retrieve function type signature
        let params = self
            .params
            .iter()
            .map(|type_var| type_var.signature.clone())
            .collect::<Types>();

        let ret = self.ret.clone();
        let fn_type = FnType { params, ret };

        let fn_type_sig = fn_type.type_of(scope)?;
        let var = Scope::Var::build_var(&self.id, &fn_type_sig);
        let _ = scope.register_var(var)?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventDef {
    type Output = ();
    type Context = ();
    fn resolve(
        &self,
        _scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
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
    fn resolve(
        &self,
        _scope: &mut Scope,
        _context: &Self::Context,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
