use std::cell::Ref;

use super::Scope;
use crate::ast::statements::return_stat::Return;
use crate::ast::statements::Statement;
use crate::e_static;
use crate::semantic::scope::static_types::StaticType;
use crate::semantic::scope::user_type_impl::UserType;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::ScopeApi, Either, Resolve, SemanticError, TypeOf};
use crate::semantic::{EType, MergeType};

impl<OuterScope: ScopeApi> TypeOf<OuterScope> for Scope<OuterScope> {
    fn type_of(&self, _scope: &Ref<OuterScope>) -> Result<EType, SemanticError>
    where
        OuterScope: ScopeApi,
        Self: Sized + Resolve<OuterScope>,
    {
        let binding = self.inner_scope.borrow();
        let Some(binding) = binding.as_ref() else {
            return Err(SemanticError::NotResolvedYet);
        };
        let inner_scope = binding.borrow();
        let mut return_type = e_static!(<StaticType as BuildStaticType<OuterScope>>::build_unit());

        for instruction in &self.instructions {
            match instruction {
                Statement::Flow(value) => {
                    let value_type = value.type_of(&inner_scope)?;
                    return_type = return_type.merge(&value_type, &inner_scope)?;
                }
                Statement::Loops(value) => {
                    let value_type = value.type_of(&inner_scope);
                    let value_type = value_type?;
                    return_type = return_type.merge(&value_type, &inner_scope)?;
                }
                Statement::Return(value) => {
                    if self.is_generator.get() {
                        match value {
                            Return::Expr { .. } | Return::Unit => {
                                return Err(SemanticError::CantReturn)
                            }
                            _ => {}
                        }
                    }
                    let value_type = value.type_of(&inner_scope)?;
                    return_type = return_type.merge(&value_type, &inner_scope)?;
                }
                _ => {}
            }
        }
        Ok(return_type)
    }
}
