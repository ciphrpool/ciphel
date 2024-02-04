use std::{cell::RefCell, rc::Rc};

use crate::{
    semantic::scope::ScopeApi,
    vm::{
        strips::Strip,
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::Flow;

impl<Scope: ScopeApi> GenerateCode<Scope> for Flow<Scope> {
    fn gencode(
        &self,
        scope: &Rc<RefCell<Scope>>,
        instructions: &Rc<RefCell<Vec<Strip>>>,
        offset: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
