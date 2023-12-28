use crate::semantic::{CompatibleWith, Resolve, ScopeApi, SemanticError, TypeOf};

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};

impl<Scope: ScopeApi> Resolve<Scope> for Declaration {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Declaration::Declared(value) => {
                let _ = value.resolve(scope)?;
                let _ = scope.register_var(todo!())?;

                Ok(())
            }
            Declaration::Assigned { left, right } => {
                let _ = left.resolve(scope)?;
                let _ = right.resolve(scope)?;

                let left_type = left.type_of(scope)?;
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
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.signature.resolve(scope)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for DeclaredVar {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            DeclaredVar::Id(_) => Ok(()),
            DeclaredVar::Typed(value) => value.resolve(scope),
            DeclaredVar::Pattern(value) => value.resolve(scope),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternVar {
    type Output = ();
    fn resolve(&self, scope: &Scope) -> Result<Self::Output, SemanticError>
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
