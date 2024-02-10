use std::{cell::RefCell, rc::Rc};

use crate::{
    semantic::{scope::ScopeApi, MutRc},
    vm::{
        casm::{Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::Loop;

impl<Scope: ScopeApi> GenerateCode<Scope> for Loop<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
