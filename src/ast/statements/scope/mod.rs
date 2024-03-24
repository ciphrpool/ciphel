use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

use crate::{
    ast::utils::strings::ID,
    semantic::{
        scope::{var_impl::Var, ClosureState, ScopeApi},
        AccessLevel, Metadata, MutRc, SemanticError,
    },
};

use super::Statement;

pub mod scope_gencode;
pub mod scope_parse;
pub mod scope_resolve;
pub mod scope_typeof;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<Inner: ScopeApi> {
    pub instructions: Vec<Statement<Inner>>,
    pub inner_scope: RefCell<Option<MutRc<Inner>>>,
    pub can_capture: Cell<ClosureState>,
    pub is_loop: Cell<bool>,
    pub is_yieldable: Cell<bool>,
    pub caller: MutRc<Option<Var>>,
    pub metadata: Metadata,
}

impl<InnerScope: ScopeApi> Scope<InnerScope> {
    pub fn scope(&self) -> Result<MutRc<InnerScope>, SemanticError> {
        match self.inner_scope.borrow().as_ref() {
            Some(inner) => Ok(inner.clone()),
            None => Err(SemanticError::NotResolvedYet),
        }
    }

    // pub fn env_vars(&self) -> Result<&Vec<Rc<Var>>, SemanticError> {
    //     match self.inner_scope.borrow().as_ref() {
    //         Some(inner) => Ok(inner.as_ref().borrow().env_vars()),
    //         None => Err(SemanticError::NotResolvedYet),
    //     }
    // }

    // // pub fn parameters_size(&self) -> Result<usize, SemanticError> {
    // //     match self.inner_scope.borrow().as_ref() {
    // //         Some(inner) => Ok(inner.as_ref().borrow().parameters_size()),
    // //         None => Err(SemanticError::NotResolvedYet),
    // //     }
    // // }
    pub fn set_caller(&self, caller: Var) {
        *self.caller.as_ref().borrow_mut() = Some(caller);
    }

    pub fn to_capturing(&self, state: ClosureState) {
        self.can_capture.set(state);
    }

    pub fn to_loop(&self) {
        self.is_loop.set(true);
    }
    pub fn to_yieldable(&self) {
        self.is_yieldable.set(true);
    }
}
