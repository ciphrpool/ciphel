use std::cell::Ref;

use crate::semantic::TypeOf;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::GenerateCodePlatform;
use crate::{
    ast::expressions::Expression,
    semantic::{scope::ScopeApi, EType, MutRc, Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadFn {
    Spawn,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadCasm {
    Spawn,
    Close,
}
impl ThreadFn {
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::SPAWN => Some(ThreadFn::Spawn),
            lexem::CLOSE => Some(ThreadFn::Close),
            _ => None,
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for ThreadFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression<Scope>>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            ThreadFn::Spawn => todo!(),
            ThreadFn::Close => todo!(),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for ThreadFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            ThreadFn::Spawn => todo!(),
            ThreadFn::Close => todo!(),
        }
    }
}

impl<Scope: ScopeApi> GenerateCodePlatform<Scope> for ThreadFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
        params_size: usize,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
