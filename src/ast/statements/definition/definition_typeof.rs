use std::cell::Ref;

use super::{EventCondition, EventDef};

use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType},
    Resolve, SemanticError, TypeOf,
};

impl TypeOf for EventDef {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        todo!()
    }
}

impl TypeOf for EventCondition {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<crate::semantic::EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        todo!()
    }
}
