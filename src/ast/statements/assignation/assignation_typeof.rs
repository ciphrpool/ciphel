use std::cell::Ref;

use super::{AssignValue, Assignation};
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::static_types::StaticType, Resolve, SemanticError, TypeOf};
use crate::semantic::{EType, Either};

impl TypeOf for Assignation {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType>::build_unit().into(),
        ))
    }
}
impl TypeOf for AssignValue {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            AssignValue::Scope(value) => value.type_of(&scope),
            AssignValue::Expr(value) => value.type_of(&scope),
        }
    }
}
