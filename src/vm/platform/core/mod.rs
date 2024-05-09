use std::cell::Ref;

use crate::semantic::scope::scope::Scope;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::stdio::StdIO;
use crate::vm::vm::CasmMetadata;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError, TypeOf},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, Executable, GenerateCode, RuntimeError},
    },
};

use self::{
    alloc::{AllocCasm, AllocFn},
    thread::{ThreadCasm, ThreadFn},
};

pub mod alloc;
pub mod thread;

#[derive(Debug, Clone, PartialEq)]
pub enum CoreFn {
    Alloc(AllocFn),
    Thread(ThreadFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreCasm {
    Alloc(AllocCasm),
    Thread(ThreadCasm),
}

impl<G: crate::GameEngineStaticFn + Clone> CasmMetadata<G> for CoreCasm {
    fn name(&self, stdio: &mut StdIO<G>, program: &CasmProgram, engine: &mut G) {
        match self {
            CoreCasm::Alloc(value) => value.name(stdio, program, engine),
            CoreCasm::Thread(value) => value.name(stdio, program, engine),
        }
    }
}

impl CoreFn {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        if let Some(value) = AllocFn::from(suffixe, id) {
            return Some(CoreFn::Alloc(value));
        }
        if let Some(value) = ThreadFn::from(suffixe, id) {
            return Some(CoreFn::Thread(value));
        }
        None
    }
}

impl Resolve for CoreFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            CoreFn::Alloc(value) => value.resolve(scope, context, extra),
            CoreFn::Thread(value) => value.resolve(scope, context, extra),
        }
    }
}

impl TypeOf for CoreFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            CoreFn::Alloc(value) => value.type_of(scope),
            CoreFn::Thread(value) => value.type_of(scope),
        }
    }
}

impl GenerateCode for CoreFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            CoreFn::Alloc(value) => value.gencode(scope, instructions),
            CoreFn::Thread(value) => value.gencode(scope, instructions),
        }
    }
}

impl<G: crate::GameEngineStaticFn + Clone> Executable<G> for CoreCasm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO<G>,
        engine: &mut G,
    ) -> Result<(), RuntimeError> {
        match self {
            CoreCasm::Alloc(value) => value.execute(program, stack, heap, stdio, engine),
            CoreCasm::Thread(value) => value.execute(program, stack, heap, stdio, engine),
        }
    }
}
