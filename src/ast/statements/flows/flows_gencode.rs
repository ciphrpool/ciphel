use std::{cell::RefCell, rc::Rc};

use crate::{
    semantic::{scope::ScopeApi, MutRc},
    vm::{
        casm::{Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{CallStat, Flow};

impl<Scope: ScopeApi> GenerateCode<Scope> for Flow<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Flow::If(_) => todo!(),
            Flow::Match(_) => todo!(),
            Flow::Try(_) => todo!(),
            Flow::Call(value) => value.gencode(scope, instructions),
        }
    }
}

impl<Scope: ScopeApi> GenerateCode<Scope> for CallStat<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        self.call.gencode(scope, instructions)
    }
}
