use std::{cell::RefCell, rc::Rc};

use crate::semantic::scope::ScopeApi;

use super::Statement;

pub mod scope_parse;
pub mod scope_resolve;
pub mod scope_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<Inner: ScopeApi> {
    pub instructions: Vec<Statement<Inner>>,
    pub inner_scope: RefCell<Option<Rc<RefCell<Inner>>>>,
}
