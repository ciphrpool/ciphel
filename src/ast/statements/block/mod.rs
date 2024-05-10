use std::cell::{Cell, RefCell};

use crate::semantic::{
    scope::{var_impl::Var, ClosureState},
    Metadata, MutRc, SemanticError,
};

use super::Statement;
use crate::semantic::scope::scope::Scope;

pub mod block_gencode;
pub mod block_parse;
pub mod block_resolve;
pub mod block_typeof;

#[derive(Debug, Clone)]
pub struct Block {
    pub instructions: Vec<Statement>,
    pub inner_scope: RefCell<Option<MutRc<Scope>>>,
    pub can_capture: Cell<ClosureState>,
    pub is_loop: Cell<bool>,
    pub caller: MutRc<Option<Var>>,
    pub metadata: Metadata,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
            && self.can_capture == other.can_capture
            && self.is_loop == other.is_loop
            && self.caller == other.caller
            && self.metadata == other.metadata
    }
}

impl Block {
    pub fn scope(&self) -> Result<MutRc<Scope>, SemanticError> {
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
}
