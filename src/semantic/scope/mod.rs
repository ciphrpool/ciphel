use crate::ast::statements::definition;

use self::{static_types::NumberType, user_types::UserType};

use super::SemanticError;
pub mod scope;
pub mod static_types;
pub mod user_types;

pub trait BuildUserType {
    fn build_usertype(
        type_sig: &definition::TypeDef,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<UserType, SemanticError>;
}
