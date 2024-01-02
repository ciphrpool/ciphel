use super::Scope;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{CompatibleWith, MergeType, TypeOf};
use crate::{
    ast::statements::Statement,
    semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError},
};

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope {
    type Output = ();
    type Context = Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>;
    fn resolve(&self, scope: &mut OuterScope, context: &Self::Context) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        let mut inner_scope = scope.child_scope()?;

        for instruction in &self.instructions {
            let _ = instruction.resolve(&mut inner_scope, context)?;
        }
        let return_type = self.type_of(scope)?;
        let _ = context.compatible_with(&return_type, scope)?;
        Ok(())
    }
}
