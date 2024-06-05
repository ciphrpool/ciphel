
use super::Block;
use crate::ast::statements::Statement;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::StaticType;
use crate::{arw_read, e_static};

use crate::semantic::scope::BuildStaticType;
use crate::semantic::{EType, MergeType};
use crate::semantic::{Resolve, SemanticError, TypeOf};

impl TypeOf for Block {
    fn type_of(&self, _scope: &std::sync::RwLockReadGuard<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        let binding = &self.inner_scope;
        let Some(binding) = binding else {
            return Err(SemanticError::NotResolvedYet);
        };
        let inner_scope = arw_read!(binding, SemanticError::ConcurrencyError)?;
        let mut return_type = e_static!(<StaticType as BuildStaticType>::build_unit());

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
                    let value_type = value.type_of(&inner_scope)?;
                    return_type = return_type.merge(&value_type, &inner_scope)?;
                }
                _ => {}
            }
        }
        Ok(return_type)
    }
}
