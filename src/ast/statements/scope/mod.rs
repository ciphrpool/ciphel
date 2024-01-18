use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::utils::strings::ID,
    semantic::{scope::ScopeApi, SemanticError},
};

use super::Statement;

pub mod scope_parse;
pub mod scope_resolve;
pub mod scope_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<Inner: ScopeApi> {
    pub instructions: Vec<Statement<Inner>>,
    pub inner_scope: RefCell<Option<Rc<RefCell<Inner>>>>,
}

impl<InnerScope: ScopeApi> Scope<InnerScope> {
    pub fn find_outer_vars(&self) -> Result<HashMap<ID, Rc<InnerScope::Var>>, SemanticError> {
        match self.inner_scope.borrow().as_ref() {
            Some(inner) => Ok(inner.as_ref().borrow().find_outer_vars()),
            None => Err(SemanticError::NotResolvedYet),
        }
    }
}
