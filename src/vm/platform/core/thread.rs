use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::semantic::TypeOf;
use crate::vm::platform::utils::lexem;
use crate::vm::vm::CasmMetadata;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError},
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

impl CasmMetadata for ThreadCasm {
    fn name(&self, stdio: &mut crate::vm::stdio::StdIO, program: &CasmProgram) {
        match self {
            ThreadCasm::Spawn => stdio.push_casm_lib("spawn_with_scope"),
            ThreadCasm::Close => stdio.push_casm_lib("close"),
        }
    }

    fn weight(&self) -> usize {
        todo!()
    }
}
impl ThreadFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if suffixe != lexem::CORE {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::SPAWN => Some(ThreadFn::Spawn),
            lexem::CLOSE => Some(ThreadFn::Close),
            _ => None,
        }
    }
}
impl Resolve for ThreadFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            ThreadFn::Spawn => todo!(),
            ThreadFn::Close => todo!(),
        }
    }
}
impl TypeOf for ThreadFn {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            ThreadFn::Spawn => todo!(),
            ThreadFn::Close => todo!(),
        }
    }
}

impl GenerateCode for ThreadFn {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        todo!()
    }
}
