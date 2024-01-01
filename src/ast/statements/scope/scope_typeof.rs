use super::Scope;
use crate::ast::statements::Statement;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::MergeType;
use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf};

impl<OuterScope: ScopeApi> TypeOf<OuterScope> for Scope {
    fn type_of(
        &self,
        scope: &OuterScope,
    ) -> Result<Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>, SemanticError>
    where
        OuterScope: ScopeApi,
        Self: Sized + Resolve<OuterScope>,
    {
        let return_type: OuterScope::StaticType = OuterScope::StaticType::build_unit();
        let mut return_type = return_type.type_of(scope)?;

        for instruction in &self.instructions {
            match instruction {
                Statement::Flow(value) => {
                    let value_type = value.type_of(scope)?;
                    if let Some(value_type) = value_type {
                        return_type = return_type.merge(&value_type, scope)?;
                    }
                }
                Statement::Loops(value) => {
                    let value_type = value.type_of(scope)?;
                    if let Some(value_type) = value_type {
                        return_type = return_type.merge(&value_type, scope)?;
                    }
                }
                Statement::Return(value) => {
                    let value_type = value.type_of(scope)?;
                    if let Some(value_type) = value_type {
                        return_type = return_type.merge(&value_type, scope)?;
                    }
                }
                _ => {}
            }
        }

        Ok(return_type)
    }
}
