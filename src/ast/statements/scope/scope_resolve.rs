use std::cell::RefCell;
use std::rc::Rc;

use super::Scope;

use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError};
use crate::semantic::{CompatibleWith, TypeOf};

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope {
    type Output = ();
    type Context = Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<OuterScope>>,
        context: &Self::Context,
    ) -> Result<(), SemanticError>
    where
        Self: Sized,
        OuterScope: ScopeApi,
    {
        let mut inner_scope = OuterScope::child_scope(scope)?;

        for instruction in &self.instructions {
            let _ = instruction.resolve(&mut inner_scope, context)?;
        }
        let return_type = self.type_of(&scope.borrow())?;
        let _ = context.compatible_with(&return_type, &scope.borrow())?;
        Ok(())
    }
}
