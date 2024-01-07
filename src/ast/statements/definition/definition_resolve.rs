use super::{Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, TypeDef, UnionDef};
use crate::ast::types::FnType;
use crate::ast::types::Types;
use crate::semantic::scope::BuildUserType;
use crate::semantic::scope::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, CompatibleWith, Resolve, SemanticError, TypeOf};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Definition<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Definition::Type(value) => value.resolve(scope, &(), &()),
            Definition::Fn(value) => value.resolve(scope, &(), &()),
            Definition::Event(value) => value.resolve(scope, &(), &()),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TypeDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = match self {
            TypeDef::Struct(value) => value.resolve(scope, context, extra),
            TypeDef::Union(value) => value.resolve(scope, context, extra),
            TypeDef::Enum(value) => value.resolve(scope, context, extra),
        }?;
        let id = match &self {
            TypeDef::Struct(value) => &value.id,
            TypeDef::Union(value) => &value.id,
            TypeDef::Enum(value) => &value.id,
        };

        let mut borrowed_scope = scope.borrow_mut();
        let _ = borrowed_scope
            .register_type(id, Scope::UserType::build_usertype(self, &scope.borrow())?)?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for StructDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        for (_, type_siq) in &self.fields {
            let _ = type_siq.resolve(scope, context, extra)?;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        for (_, variant) in &self.variants {
            for (_, type_sig) in variant {
                let _ = type_sig.resolve(scope, context, extra);
            }
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EnumDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnDef<Scope> {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        for value in &self.params {
            let _ = value.resolve(scope, context, extra)?;
        }

        let _ = self.ret.resolve(scope, context, extra)?;
        let return_type = self.ret.type_of(&scope.borrow())?;

        // inner_scope.borrow_mut().attach(
        //     self.params
        //         .iter()
        //         .filter_map(|param| {
        //             param
        //                 .type_of(&scope.borrow())
        //                 .ok()
        //                 .map(|p| (param.id.clone(), p))
        //         })
        //         .map(|(id, param)| Scope::Var::build_var(&id, &param)),
        // );

        let vars = self
            .params
            .iter()
            .filter_map(|param| {
                param
                    .type_of(&scope.borrow())
                    .ok()
                    .map(|p| (param.id.clone(), p))
            })
            .map(|(id, param)| Scope::Var::build_var(&id, &param))
            .collect::<Vec<Scope::Var>>();

        let _ = return_type.compatible_with(&self.scope, &scope.borrow())?;
        let _ = self.scope.resolve(scope, &Some(return_type), &vars)?;

        // convert to FnType -> GOAL : Retrieve function type signature
        let params = self
            .params
            .iter()
            .map(|type_var| type_var.signature.clone())
            .collect::<Types>();

        let ret = self.ret.clone();
        let fn_type = FnType { params, ret };

        let fn_type_sig = fn_type.type_of(&scope.borrow())?;
        let var = Scope::Var::build_var(&self.id, &fn_type_sig);
        let _ = scope.borrow_mut().register_var(var)?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventDef<Scope> {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
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
    type Extra = ();
    fn resolve(
        &self,
        _scope: &Rc<RefCell<Scope>>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        todo!()
    }
}
