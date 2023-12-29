use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl<Scope: ScopeApi> Resolve<Scope> for Declaration {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Declaration::Declared(value) => {
                let _ = value.resolve(scope, context)?;
                let _ = scope.register_var(todo!())?;

                Ok(())
            }
            Declaration::Assigned { left, right } => {
                let _ = left.resolve(scope, context)?;
                let left_type = left.type_of(scope)?;
                let _ = right.resolve(scope, &left_type)?;

                if left_type.is_some() {
                    let _ = left_type.compatible_with(right, scope)?;
                }

                let _ = scope.register_var(todo!())?;

                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TypedVar {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.signature.resolve(scope, context)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for DeclaredVar {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            DeclaredVar::Id(_) => Ok(()),
            DeclaredVar::Typed(value) => value.resolve(scope, context),
            DeclaredVar::Pattern(value) => value.resolve(scope, context),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternVar {
    type Output = ();
    type Context = ();
    fn resolve(&self, scope: &Scope, context: &Self::Context) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            PatternVar::UnionInline {
                typename,
                variant,
                vars,
            } => todo!(),
            PatternVar::UnionFields {
                typename,
                variant,
                vars,
            } => todo!(),
            PatternVar::StructInline { typename, vars } => todo!(),
            PatternVar::StructFields { typename, vars } => todo!(),
            PatternVar::Tuple(_) => todo!(),
        }
    }
}
