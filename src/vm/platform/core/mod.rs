use crate::ast::utils::strings::ID;
use crate::semantic::scope::scope::ScopeManager;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::stdio::StdIO;
use crate::vm::vm::CasmMetadata;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Resolve, SemanticError, TypeOf},
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

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for CoreCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            CoreCasm::Alloc(value) => value.name(stdio, program, engine),
            CoreCasm::Thread(value) => value.name(stdio, program, engine),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            CoreCasm::Alloc(value) => <AllocCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Thread(value) => <ThreadCasm as CasmMetadata<G>>::weight(value),
        }
    }
}

impl CoreFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
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
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            CoreFn::Alloc(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            CoreFn::Thread(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
        }
    }
}

impl TypeOf for CoreFn {
    fn type_of(
        &self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            CoreFn::Alloc(value) => value.type_of(scope_manager, scope_id),
            CoreFn::Thread(value) => value.type_of(scope_manager, scope_id),
        }
    }
}

impl GenerateCode for CoreFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            CoreFn::Alloc(value) => value.gencode(scope_manager, scope_id, instructions, context),
            CoreFn::Thread(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}

impl<G: crate::GameEngineStaticFn> Executable<G> for CoreCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            CoreCasm::Alloc(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Thread(value) => value.execute(program, stack, heap, stdio, engine, tid),
        }
    }
}
