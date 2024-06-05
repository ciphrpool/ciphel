use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{
    arw_write,
    semantic::{
        scope::{var_impl::Var, ClosureState}, ArcRwLock, Metadata, SemanticError,
    },
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
    pub inner_scope: Option<ArcRwLock<Scope>>,
    pub can_capture: ArcRwLock<ClosureState>,
    pub is_loop: Arc<AtomicBool>,
    pub caller: ArcRwLock<Option<Var>>,
    pub metadata: Metadata,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
            && self
                .can_capture
                .read()
                .map(|value| value.clone())
                .ok()
                .unwrap_or(ClosureState::DEFAULT)
                == other
                    .can_capture
                    .read()
                    .map(|value| value.clone())
                    .ok()
                    .unwrap_or(ClosureState::DEFAULT)
            && self.is_loop.load(std::sync::atomic::Ordering::Acquire)
                == other.is_loop.load(std::sync::atomic::Ordering::Acquire)
            && self
                .caller
                .read()
                .map(|value| value.clone())
                .ok()
                .unwrap_or(None)
                == other
                    .caller
                    .read()
                    .map(|value| value.clone())
                    .ok()
                    .unwrap_or(None)
            && self.metadata == other.metadata
    }
}

impl Block {
    pub fn scope(&self) -> Result<ArcRwLock<Scope>, SemanticError> {
        match &self.inner_scope {
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
    pub fn set_caller(&self, caller: Var) -> Result<(), SemanticError> {
        let mut self_caller = arw_write!(self.caller, SemanticError::ConcurrencyError)?;
        *self_caller = Some(caller);
        Ok(())
    }

    pub fn to_capturing(&self, state: ClosureState) -> Result<(), SemanticError> {
        let mut can_capture = arw_write!(self.can_capture, SemanticError::ConcurrencyError)?;
        *can_capture = state;
        Ok(())
    }

    pub fn to_loop(&self) {
        self.is_loop.store(true, Ordering::Release);
    }
}
