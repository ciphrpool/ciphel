use std::fmt::Debug;

use crate::ast::{statements::definition, types, utils::strings::ID};

use self::{
    static_types::{NumberType, StaticType},
    user_type_impl::UserType,
};
use crate::semantic::scope::scope::ScopeManager;

use super::{EType, SemanticError};
pub mod scope;
pub mod static_types;
pub mod user_type_impl;

pub trait BuildUserType {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<UserType, SemanticError>;
}
