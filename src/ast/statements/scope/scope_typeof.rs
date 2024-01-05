use std::cell::Ref;

use super::Scope;
use crate::ast::statements::Statement;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::MergeType;
use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf};

impl<OuterScope: ScopeApi> TypeOf<OuterScope> for Scope {
    fn type_of(
        &self,
        scope: &Ref<OuterScope>,
    ) -> Result<EitherType<OuterScope::UserType, OuterScope::StaticType>, SemanticError>
    where
        OuterScope: ScopeApi,
        Self: Sized + Resolve<OuterScope>,
    {
        let mut return_type = EitherType::Static(OuterScope::StaticType::build_unit());

        for instruction in &self.instructions {
            match instruction {
                Statement::Flow(value) => {
                    let value_type = value.type_of(&scope)?;
                    return_type = return_type.merge(&value_type, scope)?;
                }
                Statement::Loops(value) => {
                    let value_type = value.type_of(&scope)?;
                    return_type = return_type.merge(&value_type, scope)?;
                }
                Statement::Return(value) => {
                    let value_type = value.type_of(&scope)?;
                    return_type = return_type.merge(&value_type, scope)?;
                }
                _ => {}
            }
        }
        Ok(return_type)
    }
}
