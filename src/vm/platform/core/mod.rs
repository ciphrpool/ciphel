use core_map::{MapCasm, MapFn};
use core_string::{StringCasm, StringFn};
use core_vector::{VectorCasm, VectorFn};

use crate::ast::utils::strings::ID;
use crate::semantic::ResolvePlatform;
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

pub mod alloc;
pub mod core_map;
pub mod core_string;
pub mod core_vector;
pub mod thread;

use self::{
    alloc::{AllocCasm, AllocFn},
    thread::{ThreadCasm, ThreadFn},
};

#[derive(Debug, Clone, PartialEq)]
pub enum CoreFn {
    Vec(VectorFn),
    Map(MapFn),
    String(StringFn),
    Alloc(AllocFn),
    Thread(ThreadFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreCasm {
    Vec(VectorCasm),
    Map(MapCasm),
    String(StringCasm),
    Alloc(AllocCasm),
    Thread(ThreadCasm),
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for CoreCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            CoreCasm::Alloc(value) => value.name(stdio, program, engine),
            CoreCasm::Vec(value) => value.name(stdio, program, engine),
            CoreCasm::Map(value) => value.name(stdio, program, engine),
            CoreCasm::String(value) => value.name(stdio, program, engine),
            CoreCasm::Thread(value) => value.name(stdio, program, engine),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            CoreCasm::Alloc(value) => <AllocCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Thread(value) => <ThreadCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Vec(value) => <VectorCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::Map(value) => <MapCasm as CasmMetadata<G>>::weight(value),
            CoreCasm::String(value) => <StringCasm as CasmMetadata<G>>::weight(value),
        }
    }
}

impl CoreFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        if let Some(value) = VectorFn::from(suffixe, id) {
            return Some(CoreFn::Vec(value));
        }
        if let Some(value) = MapFn::from(suffixe, id) {
            return Some(CoreFn::Map(value));
        }
        if let Some(value) = StringFn::from(suffixe, id) {
            return Some(CoreFn::String(value));
        }
        if let Some(value) = AllocFn::from(suffixe, id) {
            return Some(CoreFn::Alloc(value));
        }
        if let Some(value) = ThreadFn::from(suffixe, id) {
            return Some(CoreFn::Thread(value));
        }
        None
    }
}

impl ResolvePlatform for CoreFn {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            CoreFn::Alloc(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, parameters)
            }
            CoreFn::Thread(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, parameters)
            }
            CoreFn::Vec(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            CoreFn::Map(value) => value.resolve::<G>(scope_manager, scope_id, context, parameters),
            CoreFn::String(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, parameters)
            }
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
            CoreFn::Vec(value) => value.gencode(scope_manager, scope_id, instructions, context),
            CoreFn::String(value) => value.gencode(scope_manager, scope_id, instructions, context),
            CoreFn::Map(value) => value.gencode(scope_manager, scope_id, instructions, context),
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
            CoreCasm::String(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Map(value) => value.execute(program, stack, heap, stdio, engine, tid),
            CoreCasm::Vec(value) => value.execute(program, stack, heap, stdio, engine, tid),
        }
    }
}
